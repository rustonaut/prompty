use std::env;
use crate::iface::{ColumnCountPlugin, ErrorMessage};


pub mod git;
pub mod cwd;
pub mod terminal;

pub use self::{
    git::Git,
    cwd::CwdPath,
    terminal::Terminal
};



pub struct ColumnCount;

impl ColumnCountPlugin for ColumnCount {
    fn get_column_count() -> Result<usize, ErrorMessage> {
        let first_relevant_arg = env::args_os().skip(1).next();
        if let Some(os_arg) = first_relevant_arg {
            if let Some(str_arg) = os_arg.to_str() {
                if let Ok(count) = str_arg.parse() {
                    return Ok(count);
                }
            }

            let err = format!("invalid column count arg: {}", os_arg.to_string_lossy());
            Err(ErrorMessage::new(err))
        } else {
            let err = "missing column count argument";
            Err(ErrorMessage::new(err))
        }
    }
}
