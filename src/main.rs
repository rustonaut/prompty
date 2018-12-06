use std::io::{stdout, Write};

use crate::{
    git_state::{cwd_git_status, GitStatus},
    relevant_path::cwd_relevant_path
};


mod config;
mod git_state;
mod relevant_path;

const MIN_WIDTH: usize = config::MIN_CONTENT_WIDTH+2;

fn main() {
    let mut out = String::with_capacity(MIN_WIDTH*3);
    let path = cwd_relevant_path();
    let git_status = cwd_git_status();

    write_prompt(&mut out, &path, git_status);

    let stdout = stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(out.as_ref()).unwrap();
    stdout.flush().unwrap();
}

fn write_prompt(out: &mut String, path_text: &str, git_status: Option<GitStatus>) {
    out.push(config::CORNER_SE);
    write_path_status(out, path_text);

    match git_status {
        None => {
            out.push(config::LINE);
        },
        Some(status) => {
            out.push(config::CORNER_SW);
            out.push('\n');
            out.push(config::CORNER_NSE);
            write_git_status(out, status);
            out.push(config::CORNER_NW);
        }
    }

    out.push('\n');
    out.push(config::CORNER_NE);
    write_input_prompt(out);
}

fn write_path_status(out: &mut String, path_text: &str) {
    let offset = out.len();
    out.push(config::LINE);
    out.push(config::TEXT_START);
    out.push_str(path_text);
    out.push(config::TEXT_END);
    write_fill_line(out, offset, config::MIN_CONTENT_WIDTH);
}

fn write_git_status(out: &mut String, git_status: GitStatus) {
    let offset = out.len();
    out.push(config::LINE);
    out.push(config::TEXT_START);
    out.push_str(git_status.branch());
    out.push(config::TEXT_END);
    out.push(config::TEXT_START);
    out.push(git_status.status_symbol());
    out.push(config::TEXT_END);
    write_fill_line(out, offset, config::MIN_CONTENT_WIDTH);
}

fn write_fill_line(out: &mut String, offset: usize, len: usize) {
    let mut line_len = out[offset..].chars().count();
    while line_len < len {
        out.push(config::LINE);
        line_len += 1;
    }
}

fn write_input_prompt(out: &mut String) {
    out.push(config::LINE);
    out.push(config::INPUT_START);
}


