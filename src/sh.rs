use std::{
    ffi::OsStr,
    fs,
    io::{IsTerminal, Write},
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use miette::{miette, Context, IntoDiagnostic, Result};
use tracing::debug;

/// All args must be accepted in OsStr::new.
///
/// This macro will contruct an array of OsStr from all the args.
///
/// Designed to be used to inline args:
///
/// ```rust
/// // let out = PathBuf::from("/tmp/acme.index");
/// // let url = OsString::from("https://acme.com/");
/// // tsp::sh::exec("echo", args!["hello", out.as_ref()]);
/// ```
macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        vec![
            $(
                OsStr::new($arg)
            ),*
        ]
    };
}

/// Execute a command as if it was part of the current program,
pub fn exec<C, I, S>(command: C, args: I) -> Result<Output>
where
    C: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(&command);
    cmd.args(args);
    let output = cmd
        .output()
        .into_diagnostic()
        .context(format!("Failed to execute {:?}.", command.as_ref()))?;
    debug!("exec {:?}", cmd);
    if output.status.success() {
        Ok(output)
    } else {
        match output.status.code() {
            Some(code) => Err(miette!(
                "{:?} failed with exit status {}.",
                command.as_ref(),
                code
            )),
            _ => Err(miette!(
                "{:?} interrupted by signal {}.",
                command.as_ref(),
                output.status.signal().unwrap()
            )),
        }
    }
}

/// Your local hometown one-eyed which.
///
/// stdin, stdout, and stderr are ignored.
pub fn which<S>(prog: S) -> Result<PathBuf>
where
    S: AsRef<OsStr>,
{
    let output = Command::new("which")
        .arg(&prog)
        .output()
        .into_diagnostic()
        .context(format!("which {:?}", prog.as_ref()))?;
    let stdout = String::from_utf8(output.stdout)
        .into_diagnostic()
        .context("Could not read stdout in UTF-8")?;
    let mut res = PathBuf::new();
    res.push(stdout.trim());
    Ok(res)
}

pub fn chmod_x<P>(prog: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    exec("chmod", args!["+x", prog.as_ref()])
        .context(format!("chmod +x {}", prog.as_ref().display()))
}

pub fn download<O, U>(out: O, url: U) -> Result<Output>
where
    O: AsRef<OsStr>,
    U: AsRef<str>,
{
    let prog = which("curl")
        .or_else(|_| which("wget"))
        .context("Finding curl or wget on your system.")?;

    match prog
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or(miette!("Reading the available download program name"))?
    {
        "curl" => exec(prog, args!["-o", out.as_ref(), "-L", url.as_ref()]),
        "wget" => exec(prog, args!["-O", out.as_ref(), url.as_ref()]),
        _ => Err(miette!("Unknown program {}", prog.to_str().unwrap_or(""))),
    }
}

pub fn gunzip<P>(gz: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    exec("gunzip", args![gz.as_ref()])
}

pub fn git_column<S, I>(input: S, indent: I, width: usize) -> Result<Output>
where
    S: AsRef<str>,
    I: AsRef<str>,
{
    let mut child = Command::new("git")
        .arg("column")
        .arg(format!(
            "--mode={}",
            if std::io::stdout().is_terminal() {
                "always"
            } else {
                "never"
            }
        ))
        .arg(format!("--indent={}", indent.as_ref()))
        .arg(format!("--width={}", width))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
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

pub fn git_clone_fast<L, R, T>(repo: R, tag: T, local: L) -> Result<()>
where
    L: AsRef<Path>,
    R: AsRef<str>,
    T: AsRef<str>,
{
    if local
        .as_ref()
        .try_exists()
        .into_diagnostic()
        .context("Checking if tree-sitter sources exist")?
    {
        exec(
            "git",
            args!["-C", local.as_ref(), "reset", "--hard", "HEAD"],
        )
        .context(format!("git -C {:?} reset --hard HEAD", local.as_ref()))?;
    } else {
        fs::create_dir_all(local.as_ref())
            .into_diagnostic()
            .context(format!("Creating local {} clone", repo.as_ref()))?;
        exec("git", args!["-C", local.as_ref(), "init"])
            .context(format!("git -C {:?} init", local.as_ref()))?;
        exec(
            "git",
            args![
                "-C",
                local.as_ref(),
                "remote",
                "add",
                "origin",
                repo.as_ref(),
            ],
        )
        .context(format!(
            "git -C {:?} remote add origin {}",
            local.as_ref(),
            repo.as_ref()
        ))?;
    }
    exec(
        "git",
        args![
            "-C",
            local.as_ref(),
            "fetch",
            "origin",
            "--depth",
            "1",
            tag.as_ref()
        ],
    )
    .context(format!(
        "git -C {:?} fetch origin --depth 1 {}",
        local.as_ref(),
        tag.as_ref()
    ))?;
    exec(
        "git",
        args!["-C", local.as_ref(), "reset", "--hard", "FETCH_HEAD"],
    )
    .context(format!(
        "git -C {:?} reset --hard FETCH_HEAD",
        local.as_ref()
    ))?;
    Ok(())
}
