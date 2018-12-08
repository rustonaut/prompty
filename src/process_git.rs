
use crate::{
    iface::{TerminalPlugin, GitPlugin, GitInfo, FormatLike, WithNotAvailableVariant}
};

const ERR_SCOPE: &str = "git";

pub fn process_git<GIT, T>(terminal: &mut T)
    where GIT: GitPlugin, T: TerminalPlugin
{
    let status =
        match GIT::lookup_status() {
            Ok(status) => status,
            Err(err) => {
                match err {
                    WithNotAvailableVariant::Err(err) => {
                        terminal.add_error_segment(ERR_SCOPE, err.msg());
                    },
                    WithNotAvailableVariant::NotAvailable => {}
                }
                return;
            }
        };

    let GitInfo { branch, has_untracked_files, has_unstaged_files, has_staged_files } = status;
    terminal.add_text_segment(&branch, FormatLike::Text);

    let (text, fmt_arg) =
        match (has_untracked_files, has_unstaged_files, has_staged_files) {
            (false, false, false) => ("++", FormatLike::ExplicitOk),
            (false, false, true ) => ("_A", FormatLike::Text),
            (false, true,  false) => ("M_", FormatLike::Text),
            (false, true,  true ) => ("AM", FormatLike::SoftWarning),
            (true,  false, false) => ("??", FormatLike::SoftWarning),
            _ => ("!!", FormatLike::HardWarning),
        };

    terminal.extend_previous_segment(text, fmt_arg);
}