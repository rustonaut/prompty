use std::{
    process::Command,
    io::BufRead
};

use crate::iface::{GitInfo, GitPlugin, ErrorMessage, WithNotAvailableVariant};

const UNMODIFIED_SHORT_STATUS_CODE: u8 = b' ';

pub struct Git;

impl GitPlugin for Git {
    fn lookup_status() -> Result<GitInfo, WithNotAvailableVariant<ErrorMessage>> {
        let output_res = Command::new("git")
            .args(&["status", "-s", "-b"])
            .output();

        let output =
            match output_res {
                Ok(output) => output,
                Err(err) => {
                    let err = ErrorMessage::new(format!("{}", err));
                    return Err(WithNotAvailableVariant::Err(err));
                }
            };

        if let Some(128) = output.status.code() {
            if output.stderr.starts_with(b"fatal: not a git repository") {
                return Err(WithNotAvailableVariant::NotAvailable);
            }
        }

        if output.status.success() {
            parse_git_info(&output.stdout)
        } else {
            let err = ErrorMessage::new(String::from_utf8_lossy(&output.stderr));
            Err(WithNotAvailableVariant::Err(err))
        }
    }
}

fn parse_git_info(stdout: &[u8]) -> Result<GitInfo, WithNotAvailableVariant<ErrorMessage>> {

    let mut branch = String::new();
    let mut has_untracked = false;
    let mut has_unstaged = false;
    let mut has_staged = false;
    let mut first = true;

    for line in stdout.lines() {
        let line = line?;

        if first {
            first = false;
            branch = parse_branch(line)?;
            continue;
        }

        let (n_has_untracked, n_has_staged, n_has_unstaged) =
            parse_status_line(line)?;

        has_untracked |= n_has_untracked;
        has_unstaged  |= n_has_unstaged;
        has_staged    |= n_has_staged;

        if has_untracked && has_unstaged && has_staged {
            break;
        }
    }

    Ok(GitInfo {
        branch,
        has_untracked_files: has_untracked,
        has_unstaged_files: has_unstaged,
        has_staged_files: has_staged
    })
}

fn parse_branch(mut line: String) -> Result<String, ErrorMessage> {
    if !line.starts_with("## ") {
        return Err(ErrorMessage::new(format!("invalid head `git status -sb` line: {}", line)));
    }

    let end_branch_name_idx = position_triple_dot(&line)
        .unwrap_or(line.len());

    // cut out unneeded parts from branch name,
    //  on a commit-less repository is will result in `No commits yet on master`
    //  which is fine
    line.truncate(end_branch_name_idx);
    line.drain(0..3);

    Ok(line)
}

fn position_triple_dot(line: &str) -> Option<usize> {
    let mut dot_count = 0;
    let res = line.bytes()
        .position(|bch| {
            if bch == b'.' {
                dot_count += 1;
                dot_count == 3
            } else {
                dot_count = 0;
                false
            }
        })
        .map(|pos| pos - 2);

    res
}

/// Returns (has_untracked, has_unstaged, has_staged)
fn parse_status_line(line: impl AsRef<str>) -> Result<(bool, bool, bool), ErrorMessage> {
    let line = line.as_ref();

    if line.len() < 3 {
        return Err(ErrorMessage::new(format!("invalid `git status -sb` line: {}", line)));
    }
    if line.as_bytes()[2] != b' ' {
        return Err(ErrorMessage::new(format!("invalid `git status -sb` line: {}", line)));
    }
    let line = &line.as_bytes()[0..2];

    if line == b"??" {
        return Ok((true, false, false));
    }

    let has_staged = line[0] != UNMODIFIED_SHORT_STATUS_CODE;
    let has_unstaged = line[1] != UNMODIFIED_SHORT_STATUS_CODE;

    Ok((false, has_staged, has_unstaged))
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_status_line() {
        assert_eq!((true,  false, false), parse_status_line("?? file").unwrap());
        assert_eq!((false, true,  false), parse_status_line("A  file").unwrap());
        assert_eq!((false, false, true ), parse_status_line(" M file").unwrap());
        assert_eq!((false, true,  false), parse_status_line("D  file").unwrap());
        assert_eq!((false, false, true ), parse_status_line(" D file").unwrap());
        assert_eq!((false, true,  false), parse_status_line("R  file").unwrap());
        assert_eq!((false, true,  true ), parse_status_line("AM file").unwrap());
        assert_eq!((false, true,  true ), parse_status_line("RM file").unwrap());
        assert_eq!((false, true,  true ), parse_status_line("RD file").unwrap());
        assert_eq!((false, true,  true ), parse_status_line("AD file").unwrap());
        assert_eq!((false, true,  true ), parse_status_line("DM file").unwrap());
        // treat any unknown status codes as "modified"
        assert_eq!((false, true,  true ), parse_status_line("XY hy there").unwrap());
    }

    #[test]
    fn parse_branch_line() {
        assert_eq!("No commits yet on master", parse_branch("## No commits yet on master".into()).unwrap());
        assert_eq!("master", parse_branch("## master...origin/master".into()).unwrap());
        assert_eq!("0.3", parse_branch("## 0.3...origin/master".into()).unwrap());
        assert_eq!("master-not_real", parse_branch("## master-not_real...origin/master".into()).unwrap());
    }
}