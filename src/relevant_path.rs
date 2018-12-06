use std::{
    env,
    ffi::OsString,
    path::PathBuf
};

use crate::config;

pub fn cwd_relevant_path() -> String {
    let top_path = env_get_top_path();
    if let Ok(dir) = env::current_dir() {
        if let Some(prefix) = top_path {
            if let Ok(suffix) = dir.strip_prefix(prefix) {
                return format!("{}", suffix.display());
            }
        }
        return format!("{}", dir.display());
    } else {
        "??????????".to_owned()
    }
}

fn env_get_top_path() -> Option<PathBuf> {
    env::var_os(config::PATH_TOP_ENV_VAR).filter(not_empty)
        .or_else(|| env::var_os("HOME").filter(not_empty))
        .map(PathBuf::from)
}


fn not_empty(var: &OsString) -> bool {
    var.len() > 0
}