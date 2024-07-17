use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use miette::{miette, Context, IntoDiagnostic, Result};

use crate::{cli, sh};

fn tag<R, V>(repo: R, version: V) -> Result<String>
where
    R: AsRef<str>,
    V: AsRef<str>,
{
    let output = sh::exec("git", ["ls-remote", "--refs", "--tags", repo.as_ref()])
        .context(format!("Fetching remote tags from {}", repo.as_ref()))?;
    let stdout = String::from_utf8(output.stdout)
        .into_diagnostic()
        .context("Could not read stdout in UTF-8")?;
    let mut tags = HashSet::new();
    for line in stdout.lines() {
        if let Some(tag) = line.split('/').last() {
            tags.insert(tag.to_string());
        }
    }
    let query = HashSet::from([
        version.as_ref().to_string(),
        format!("v{}", version.as_ref()),
    ])
    .into_iter()
    .collect();
    tags.intersection(&query).next().cloned().ok_or(miette!(
        "Finding the requested version {} in  {}",
        version.as_ref(),
        repo.as_ref()
    ))
}

fn cli<O, P, R, T>(repo: R, tag: T, platform: P, build_dir: O) -> Result<PathBuf>
where
    O: AsRef<Path>,
    P: AsRef<str>,
    R: AsRef<str>,
    T: AsRef<str>,
{
    let cli = format!("tree-sitter-{}", platform.as_ref());
    let res = PathBuf::new().join(&build_dir).join(&cli);

    if !res.exists() {
        let gz_basename = format!("{}.gz", cli);
        let url = format!(
            "{}/releases/download/{}/{}",
            repo.as_ref(),
            tag.as_ref(),
            gz_basename,
        );
        let gz = PathBuf::new().join(&build_dir).join(gz_basename);

        // TODO: show stderr in the error message, but that means
        // developing a custom error for `sh::` and implementing
        // its display and wahtnot. Check out thiserror.
        sh::download(&gz, url).context("Downloading tree-sitter-cli")?;
        sh::gunzip(gz).context("Unzipping tree-siter-cli")?;
        sh::chmod_x(&res).context("chmod +x tree-sitter-cli")?;
    }

    Ok(res)
}

/// Build tree-sitter in [`Args::build_dir`] by:
///
/// 1. performing a fast shallow clone if the documentation.
/// 1. removing the static or dynamic libraries, depending on the value of [`Args::build_static`].
pub fn build(args: &cli::BuildCommand) -> Result<()> {
    let build_dir = &args.build_dir;
    let repo = &args.tree_sitter.repo;
    let tag = tag(repo, &args.tree_sitter.version)
        .context("Figuring out the appropriate tag for requested tree-sitter version")?;
    let _cli = cli(repo, &tag, &args.tree_sitter.platform, build_dir)
        .context("Fetching tree-sitter cli for requested platform")?;
    let src = PathBuf::new().join(build_dir).join("tree-sitter");

    sh::git_clone_fast(repo, &tag, &src).context("Cloning tree-siter")?;
    sh::exec("make", args!["-C", &src]).context("Building tree-sitter")?;

    let mut lib = src.clone();
    if args.build_static {
        lib.push("libtree-sitter.a");
    } else {
        #[cfg(target_os = "linux")]
        lib.push("libtree-sitter.so");
        #[cfg(target_os = "macos")]
        lib.push("libtree-sitter.dylib");
    }
    if lib.exists() {
        fs::remove_file(&lib).into_diagnostic().context(format!(
            "Removing {} to force static linking",
            lib.display()
        ))?;
    } else if args.fresh {
        eprint!("Tried to delete {}, but couldn't find it.", lib.display());
    }

    Ok(())
}
