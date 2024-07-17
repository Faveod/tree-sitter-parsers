use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use miette::{miette, Context, IntoDiagnostic, Result};
use tracing::error;

use crate::display::Screen;
use crate::{args, cli, git, sh};

async fn tag<R, V>(repo: R, version: V) -> Result<String>
where
    R: AsRef<str>,
    V: AsRef<str>,
{
    let output = sh::exec("git", args!["ls-remote", "--refs", "--tags", repo.as_ref()])
        .await
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

async fn cli<O, P, R, T>(repo: R, tag: T, platform: P, build_dir: O) -> Result<PathBuf>
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

        sh::download(&gz, url)
            .await
            .context("Downloading tree-sitter-cli")?;
        sh::gunzip(gz).await.context("Unzipping tree-siter-cli")?;
        sh::chmod_x(&res)
            .await
            .context("chmod +x tree-sitter-cli")?;
    }

    Ok(res)
}

/// Build tree-sitter in [`Args::build_dir`] by:
///
/// 1. performing a fast shallow clone if the documentation.
/// 1. removing the static or dynamic libraries, depending on the value of [`Args::build_static`].
///
/// Also downloads the tree-sitter-cli because building it requires rust, and we don't
/// want to impose that on the consumer.
///
/// Returns the downloaded tree-sitter-cli.
pub async fn build(args: &cli::BuildCommand, screen: &mut Screen) -> Result<PathBuf> {
    let handle = screen.register("TreeSitter", 4);
    let build_dir = &args.build_dir;
    let repo = &args.tree_sitter.repo;
    let version = &args.tree_sitter.version;
    let platform = &args.tree_sitter.platform;
    handle.send(
        Some(1),
        Some(format!("Figuring out tag from version {}", version)),
    );
    let tag = tag(repo, version)
        .await
        .context("Figuring out the appropriate tag for requested tree-sitter version")?;
    handle.send(
        Some(1),
        Some(format!("Fetching tree-sitter-cli for {}", platform)),
    );
    let cli = cli(repo, &tag, platform, build_dir)
        .await
        .context("Fetching tree-sitter cli for requested platform")?;
    handle.send(Some(1), Some(format!("Building tree-sitter {}", tag)));
    let src = PathBuf::new().join(build_dir).join("tree-sitter");

    git::clone_fast(repo, &tag, &src)
        .await
        .context("Cloning tree-siter")?;
    sh::exec("make", args!["-C", &src])
        .await
        .context("Building tree-sitter")?;

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
        error!("Tried to delete {}, but couldn't find it.", lib.display());
    }

    handle.send(Some(1), Some(format!("tree-sitter {} built", tag)));
    Ok(cli)
}
