use std::{
    env,
    path::{Path, PathBuf},
};

extern crate log;

pub mod build;
pub mod cli;
pub mod config;
pub mod consts;
pub mod display;
pub mod git;
pub mod logging;
pub mod parser;
#[macro_use]
pub mod sh;
pub mod tree_sitter;

pub fn relative_to_cwd<D>(dir: D) -> PathBuf
where
    D: AsRef<Path>,
{
    let cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    let self_dir_canon = dir
        .as_ref()
        .canonicalize()
        .unwrap_or(dir.as_ref().to_path_buf());
    let disp_dir = if self_dir_canon.starts_with(&cwd) {
        dir.as_ref()
            .strip_prefix(&cwd)
            .map(|p| p.to_path_buf())
            .unwrap_or(dir.as_ref().to_path_buf())
    } else {
        self_dir_canon
    };
    disp_dir
}
