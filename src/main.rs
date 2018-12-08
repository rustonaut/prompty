extern crate terminfo;
extern crate smallvec;

use std::{
    env,
    cmp::max
};

use crate::{
    iface::*,
    process_cwd::process_cwd,
    process_git::process_git
};

mod config;
mod iface;
mod plugin_impl;
mod process_cwd;
mod process_git;



fn main() {
    if let Err(()) = try_run_alt_setup_code() {
        run_with::<
            plugin_impl::Terminal,
            plugin_impl::CwdPath,
            plugin_impl::Git,
            plugin_impl::ColumnCount
        >();
    }
}

fn run_with<TERM, PATH, GIT, COL>()
    where TERM: TerminalPlugin, PATH: CwdPathPlugin, GIT: GitPlugin, COL: ColumnCountPlugin
{
    let (columns, delayed_error) =
        match COL::get_column_count() {
            Ok(cols) => (cols, None),
            Err(err) => (config::FALLBACK_COLUMN_COUNT, Some(err))
        };

    let columns = max(columns, config::MIN_COLUMN_COUNT);

    let mut terminal = TERM::new(columns);
    if let Some(err) = delayed_error {
        terminal.add_error_segment("columns", err.msg());
    }
    process_cwd::<PATH, _>(&mut terminal);
    process_git::<GIT, _>(&mut terminal);
    terminal.flush_to_stdout(config::PROMPT_ENDING);
}

fn try_run_alt_setup_code() -> Result<(), ()> {
    let first_relevant_arg = env::args_os().skip(1).next();

    if let Some(arg) = first_relevant_arg {
        if let Some(arg) = arg.to_str() {
            if arg.starts_with("-") {
                if arg == "--bash-setup" {
                    let exec = env::current_exe().unwrap();
                    println!(r#"
                        g() {{ cd "$1"; export __PS_PATH_TOP="$(dirname $(pwd -P))"; }};
                        PS1='$("{exec}" $COLUMNS)'
                    "#, exec=exec.display());
                } else {
                    println!("{}", HELP_MSG);
                }

                return Ok(());
            }
        }
    }

    Err(())
}

const HELP_MSG: &str =
r#"usage: prompty (--bash-setup|<column_count>)

If `--bash-setup` is passed in a but of bash code
will be emmited which if passed to a `eval` call
will setup `prompty` as promt and add the `g`
command.

Note that `--bash-setup` only works well with paths
to exec which do not need any special escape sequences.
White spaces are handled.

Else if a column count is passed in, it will emit
the `prompty` promt."#;
