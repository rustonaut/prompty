use std::{
    process::Command,
    io::BufRead
};

use crate::iface::{GitInfo, GitPlugin, ErrorMessage, WithNotAvailableVariant};


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

    let end_branch_name_idx = line.bytes()
        .position(|bch| bch == b'.')
        .unwrap_or(line.len());

    // cut out unneeded parts from branch name,
    //  on a commit-less repository is will result in `No commits yet on master`
    //  which is fine
    line.truncate(end_branch_name_idx);
    let mut idx_p1 = 0;
    line.retain(|_ch| {
        idx_p1 += 1;
        idx_p1 > 3
    });

    Ok(line)
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

    let has_staged = line[0] != b' ';
    let has_unstaged = line[1] != b' ';

    Ok((false, has_staged, has_unstaged))
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_status_line() {
        assert_eq!((false, false, false), parse_status_line("## hy there").unwrap());
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
    }
}