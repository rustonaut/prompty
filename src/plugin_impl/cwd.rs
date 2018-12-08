use crate::iface::{CwdPathPlugin, ErrorMessage, WithNotAvailableVariant};

use std::{
    path::PathBuf,
    ffi::OsStr,
    env
};

use crate::config;

pub struct CwdPath;

impl CwdPathPlugin for CwdPath {
    fn get_current_path() -> Result<PathBuf, ErrorMessage> {
        env::current_dir()
            .map_err(|e| ErrorMessage::new(format!("can not access cwd: {}", e)))
    }

    fn get_top_path() -> Result<PathBuf, WithNotAvailableVariant<ErrorMessage>> {
        get_env_path(config::PATH_TOP_ENV_VAR)
    }

    fn get_home_path() -> Result<PathBuf, WithNotAvailableVariant<ErrorMessage>> {
        get_env_path("HOME")
    }
}


fn get_env_path(env_var: impl AsRef<OsStr>) -> Result<PathBuf, WithNotAvailableVariant<ErrorMessage>> {
    env::var_os(env_var)
        .ok_or_else(|| WithNotAvailableVariant::NotAvailable)
        .and_then(|osstr| {
            if osstr.is_empty() {
                Err(WithNotAvailableVariant::NotAvailable)
            } else {
                Ok(PathBuf::from(osstr))
            }
        })
}