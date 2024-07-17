use std::{
    ffi::OsStr,
    fmt::Write,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::Output,
};

use miette::{miette, Context, IntoDiagnostic, Result};
use tokio::process::Command;
use tracing::debug;

use crate::relative_to_cwd;

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
#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => {
        vec![
            $(
                OsStr::new($arg)
            ),*
        ].iter()
    };
}

/// Execute a command as if it was part of the current program.
pub async fn exec<C, I, S>(command: C, args: I) -> Result<Output>
where
    C: AsRef<OsStr>,
    I: Iterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(&command);
    cmd.args(args.clone());
    let cmd_str = display_cmd(&command, args, Option::<&OsStr>::None);
    let output = cmd
        .output()
        .await
        .into_diagnostic()
        .context(format!("exec {}.", cmd_str))?;
    debug!("exec {}", cmd_str);
    handle_exec(output, command)
}

/// Execute a command as if it was part of the current program, at a given location.
pub async fn exec_at<A, C, I, S>(at: A, command: C, args: I) -> Result<Output>
where
    A: AsRef<Path>,
    C: AsRef<OsStr>,
    I: Iterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(&command);
    cmd.args(args.clone());
    cmd.current_dir(at.as_ref());
    let cmd_str = display_cmd(&command, args, Some(at));
    let output = cmd
        .output()
        .await
        .into_diagnostic()
        .context(format!("exec {}.", cmd_str))?;
    debug!("exec_at {}", cmd_str);
    handle_exec(output, command)
}

fn handle_exec<C>(output: Output, command: C) -> std::result::Result<Output, miette::Error>
where
    C: AsRef<OsStr>,
{
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
pub async fn which<S>(prog: S) -> Result<PathBuf>
where
    S: AsRef<OsStr>,
{
    let output = Command::new("which")
        .arg(&prog)
        .output()
        .await
        .into_diagnostic()
        .context(format!("which {:?}", prog.as_ref()))?;
    let stdout = String::from_utf8(output.stdout)
        .into_diagnostic()
        .context("Could not read stdout in UTF-8")?;
    let mut res = PathBuf::new();
    res.push(stdout.trim());
    Ok(res)
}

pub async fn chmod_x<P>(prog: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    exec("chmod", args!["+x", prog.as_ref()])
        .await
        .context(format!("chmod +x {}", prog.as_ref().display()))
}

pub async fn download<O, U>(out: O, url: U) -> Result<Output>
where
    O: AsRef<OsStr>,
    U: AsRef<str>,
{
    let prog = match which("curl").await {
        Ok(path) => Ok(path),
        Err(_) => which("wget")
            .await
            .context("Finding curl or wget on your system."),
    }?;
    match prog
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or(miette!("Reading the available download program name"))?
    {
        "curl" => exec(prog, args!["-o", out.as_ref(), "-L", url.as_ref()]).await,
        "wget" => exec(prog, args!["-O", out.as_ref(), url.as_ref()]).await,
        _ => Err(miette!("Unknown program {}", prog.to_str().unwrap_or(""))),
    }
}

pub async fn gunzip<P>(gz: P) -> Result<Output>
where
    P: AsRef<Path>,
{
    exec("gunzip", args![gz.as_ref()]).await
}

// This is needlessly complicated, trying to minimize allocations, like grown-ups,
// not because it's needed —I didn't even measure anything— but becauase I'm exercising my rust.
fn display_cmd<A, C, I, S>(command: C, args: I, at: Option<A>) -> String
where
    A: AsRef<Path>,
    C: AsRef<OsStr>,
    I: Iterator<Item = S> + Clone,
    S: AsRef<OsStr>,
{
    let capacity = command.as_ref().len() + 1 // space for command
            + args.clone().map(|s| s.as_ref().len() + 1 /* spaces for args */).sum::<usize>();
    let mut res = String::with_capacity(
        capacity
            + at.as_ref().map_or(
                0,
                // + 3 = 2 brackets and a space.
                // we always overallocate by 1 (alignment aside); see the formatting of args.
                |a| a.as_ref().to_str().unwrap().len() + 3,
            ),
    );
    if let Some(ref a) = at {
        write!(res, "[{}] ", relative_to_cwd(a).to_str().unwrap()).unwrap();
    };
    write!(res, "{} ", command.as_ref().to_str().unwrap()).unwrap();
    let mut args_iter = args.enumerate();
    if let Some((_, first_arg)) = args_iter.next() {
        write!(res, "{}", first_arg.as_ref().to_str().unwrap()).unwrap();
        for (_, arg) in args_iter {
            write!(res, " {}", arg.as_ref().to_str().unwrap()).unwrap();
        }
    }
    res
}
