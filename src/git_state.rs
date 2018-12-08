use std::process::Command;

use crate::config::{
    GIT_DIRTY,
    GIT_CLEAN,
    GIT_WARN
};




pub fn cwd_git_status() -> Option<GitStatus> {
    //FIXME use git status -sb
    // with will add a line to git status with has one of following formats
    // - `## branchname`
    // - `## branchname...origin/obranchname`
    // - `## No commits yet on master` which happens in a new repo without any commits
    //    but won't happen with a new branch without any commits, maybe _except_ if the
    //    branch is parent less
    // SO:
    // - check if starts with `## No commits yet on ` if the tak tail aftet the start
    // - else take first after `## ` until `...` or linebrake
    // - Then parse lines for changes
    //
    // If I have untracked fiels + any kind of changed => strong warning (!+RED)
    // If I have untracked fiels + no changes at all => soft warn (U+0ORANGE)
    // If I have (no untracked files and) mixed changes (staged+unstages) => soft warn (P+ORANGE)
    // If I have (no untracked files and) non mixed changes (staged xor unstaged) => ok, but dirty (M+BLUE)
    // If I have no untracked  files and no changes => ok, clean (`+`+GREEN)
    ///
    let output = Command::new("git")
        .args(&["status", "-s"])
        .output()
        .unwrap();

    if !output.status.success() {
        return None;
    }

    let mut nr_untracked = 0;
    let mut nr_staged = 0;
    let mut nr_unstaged = 0;
    let mut bytes = Some(b'\n').into_iter().chain(output.stdout.iter().map(|r|*r));
    while let Some(byte) = bytes.next() {
        if byte == b'\n' {
            let next = bytes.next().unwrap_or(b' ');
            if next == b'?' {
                nr_untracked += 1;
                continue;
            }
            if next == b'A' {
                nr_staged += 1;
            }
            let next = bytes.next().unwrap_or(b' ');
            if next == b'M' {
                nr_unstaged += 1;
            }
        }
    }

    let status_symbol = status_symbol_from_git_status(nr_untracked, nr_staged, nr_unstaged);

    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();

    let branch =
        if !output.status.success() {
            "????".to_owned()
        } else {
            let len = output.stdout.len();
            String::from_utf8_lossy(&output.stdout[..len-1]).into_owned()
        };

    Some(GitStatus {
        branch,
        status_symbol
    })
}


#[derive(Debug, Clone)]
pub struct GitStatus {
    branch: String,
    status_symbol: char
}

impl GitStatus {

    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub fn status_symbol(&self) -> char {
        self.status_symbol
    }
}


fn status_symbol_from_git_status(
    nr_untracked: usize,
    nr_staged: usize,
    nr_unstaged: usize
) -> char {
    if nr_untracked != 0 {
        GIT_WARN
    } else if nr_unstaged == 0 && nr_staged == 0 {
        GIT_CLEAN
    } else {
        GIT_DIRTY
    }
}