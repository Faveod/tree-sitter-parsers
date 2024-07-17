use miette::{Context, IntoDiagnostic, Result};
use std::{ffi::OsStr, io::Write, path::Path, process};
use tokio::fs;

use crate::{args, sh::exec_at};

pub async fn clone_fast<L, R, T>(repo: R, git_ref: T, local: L) -> Result<()>
where
    L: AsRef<Path>,
    R: AsRef<str>,
    T: AsRef<str>,
{
    let cwd = local.as_ref();
    if cwd
        .try_exists()
        .into_diagnostic()
        .context("Checking if tree-sitter sources exist")?
    {
        exec_at(cwd, "git", args!["reset", "--hard", "HEAD"])
            .await
            .context("git reset --hard HEAD")?;
    } else {
        let url = repo.as_ref();
        fs::create_dir_all(cwd)
            .await
            .into_diagnostic()
            .context(format!("Creating local {} clone dir", url))?;
        exec_at(cwd, "git", args!["init"])
            .await
            .context(format!("git -C {:?} init", cwd))?;
        exec_at(cwd, "git", args!["remote", "add", "origin", url])
            .await
            .context(format!("git remote add origin {}", url))?;
    }
    let ref_ = git_ref.as_ref();
    exec_at(cwd, "git", args!["fetch", "origin", "--depth", "1", ref_])
        .await
        .context(format!("git fetch origin --depth 1 {}", ref_))?;
    exec_at(cwd, "git", args!["reset", "--hard", "FETCH_HEAD"])
        .await
        .context("git reset --hard FETCH_HEAD")?;
    Ok(())
}

pub fn column<S, I>(input: S, indent: I, width: usize) -> Result<process::Output>
where
    S: AsRef<str>,
    I: AsRef<str>,
{
    let mut child = process::Command::new("git")
        .arg("column")
        .arg("--mode=always")
        .arg(format!("--indent={}", indent.as_ref()))
        .arg(format!("--width={}", width))
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .into_diagnostic()
        .context("git column")?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_ref().as_bytes())
            .into_diagnostic()
            .context("Piping input to git column")?;
    }
    child
        .wait_with_output()
        .into_diagnostic()
        .context("Fetching output of git column")
}
