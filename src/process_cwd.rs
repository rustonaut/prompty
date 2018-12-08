const ERR_SCOPE: &str = "cwd";

use std::{
    path::{Path, PathBuf},
    ffi::OsStr
};

use crate::{
    iface::{TerminalPlugin, FormatLike, CwdPathPlugin, WithNotAvailableVariant, ErrorMessage}
};

pub(crate) fn process_cwd<CWD, T>(terminal: &mut T)
    where CWD: CwdPathPlugin, T: TerminalPlugin
{

    let base_path =
        match CWD::get_current_path() {
            Ok(path) => path,
            Err(err) => {
                terminal.add_text_segment("????", FormatLike::Text);
                terminal.add_error_segment(ERR_SCOPE, err.msg());
                return;
            }
        };

    if let Ok(()) = try_output_prefix_stripped_path(terminal, &base_path, CWD::get_top_path()) {
        return;
    }

    if let Ok(()) = try_output_prefix_stripped_path(terminal, &base_path, CWD::get_home_path()) {
        return;
    }

    output_path(terminal, &base_path, FormatLike::Text);
}

fn try_output_prefix_stripped_path(
    terminal: &mut impl TerminalPlugin,
    base_path: &Path,
    prefix: Result<PathBuf, WithNotAvailableVariant<ErrorMessage>>
) -> Result<(), ()> {
     match prefix {
        Ok(prefix) => {
            if let Ok(path) = base_path.strip_prefix(prefix) {
                if is_empty_path(path) {
                    let last = base_path.file_name().unwrap_or(OsStr::new(""));
                    output_path(terminal, Path::new(last), FormatLike::Hidden);
                } else {
                    output_path(terminal, path, FormatLike::Text);
                }
                return Ok(());
            }
        },
        Err(err) => {
            output_non_not_available_errors(terminal, &err);
        }
    }
    Err(())
}

fn is_empty_path(path: &Path) -> bool {
    path.iter().next().is_none()
}

fn output_path(out: &mut impl TerminalPlugin, path: &Path, fmt: FormatLike) {
    if let Some(str_form) = path.to_str() {
        out.add_text_segment(str_form, fmt);
    } else {
        out.add_text_segment(&format!("{}", path.display()), fmt);
    }
}

fn output_non_not_available_errors(
    out: &mut impl TerminalPlugin,
    err: &WithNotAvailableVariant<ErrorMessage>
) {
    match err {
        WithNotAvailableVariant::Err(err) => out.add_error_segment(ERR_SCOPE, err.msg()),
        WithNotAvailableVariant::NotAvailable => {}
    }
}
