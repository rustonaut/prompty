use std::{
    fmt::Debug,
    path::PathBuf,
    io
};

#[derive(Debug)]
pub struct GitInfo {
    pub branch: String,
    pub has_untracked_files: bool,
    pub has_unstaged_files: bool,
    pub has_staged_files: bool
}

/// Function which returns the git info.
///
/// If it fails because there is not git it returns `Err(None)`.
/// Else it returns `Err(Some(ErrorMessage { .. }))`.
pub trait GitPlugin {
    fn lookup_status() -> Result<GitInfo, WithNotAvailableVariant<ErrorMessage>>;
}

pub trait CwdPathPlugin {
    fn get_current_path() -> Result<PathBuf, ErrorMessage>;
    fn get_top_path() -> Result<PathBuf, WithNotAvailableVariant<ErrorMessage>>;
    fn get_home_path() -> Result<PathBuf, WithNotAvailableVariant<ErrorMessage>>;
}


/// Function which return the nr. of columns the current terminal has.
pub trait ColumnCountPlugin {
    fn get_column_count() -> Result<usize, ErrorMessage>;
}

#[derive(Debug, Copy, Clone)]
pub enum FormatLike  {
    Lines,
    Text,
    SoftWarning,
    HardWarning,
    ExplicitOk,
    Error,
    Hidden
}

pub trait TerminalPlugin: Sized + Debug {
    fn new(columns: usize) -> Self;
    fn add_text_segment(&mut self, text: &str, fmt_args: FormatLike);
    fn add_error_segment(&mut self, scope: &'static str, msg: &str);
    fn extend_previous_segment(&mut self, text: &str, fmt_args: FormatLike);
    fn flush_to_stdout(&self, prompt_ending: &str);
}


#[derive(Debug)]
pub struct ErrorMessage { msg: String }

impl ErrorMessage {
    pub fn new(msg: impl Into<String>) -> Self {
        ErrorMessage { msg: msg.into() }
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}

impl From<io::Error> for ErrorMessage {
    fn from(err: io::Error) -> Self {
        ErrorMessage::new(format!("{}", err))
    }
}

pub enum WithNotAvailableVariant<T> {
    Err(T),
    NotAvailable
}


impl<T> From<T> for WithNotAvailableVariant<T> {
    fn from(err: T) -> Self {
        WithNotAvailableVariant::Err(err)
    }
}


impl From<io::Error> for WithNotAvailableVariant<ErrorMessage> {
    fn from(err: io::Error) -> Self {
        ErrorMessage::from(err).into()
    }
}