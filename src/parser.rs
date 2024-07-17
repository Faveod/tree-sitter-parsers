use core::time;
use std::{
    env,
    fmt::{self},
    path::{Path, PathBuf},
};

use enum_dispatch::enum_dispatch;
use miette::Result;
use tracing::debug;
use url::Url;

use crate::{
    cli::{self},
    consts::TSP_FROM,
    progress,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    dir: PathBuf,
    parsers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copy {
    from: PathBuf,
    to: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clone {
    pub git_ref: Option<String>,
    pub repo: String,
    pub dir: PathBuf,
}

#[enum_dispatch(Executable)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pipe {
    Build(Build),
    Clone(Clone),
    Copy(Copy),
}

#[derive(Clone, Debug)]
pub struct Task {
    language: String,
    mailbox: progress::Handle,
    pipeline: Vec<Pipe>,
}

#[enum_dispatch]
pub trait Executable {
    fn run(&self) -> Result<()>;
}

impl Executable for Build {
    fn run(&self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }
}

impl Executable for Copy {
    fn run(&self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }
}

impl Executable for Clone {
    fn run(&self) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }
}

impl Executable for Task {
    fn run(&self) -> Result<()> {
        for p in &self.pipeline {
            self.mailbox.tick(Some(1), Some(format!("{p}")));
            p.run()?;
        }
        self.mailbox.tick(None, Some("Done"));
        Ok(())
    }
}

pub fn tasks(args: &cli::BuildCommand, screen: &mut progress::Screen) -> Vec<Task> {
    let languages = if args.languages.as_ref().map_or(true, |l| l.is_empty()) {
        args.parsers
            .as_ref()
            .map_or_else(Vec::new, |m| m.keys().cloned().collect())
    } else {
        args.languages.as_ref().unwrap().clone()
    };
    let build_dir = &args.build_dir;
    languages
        .into_iter()
        .map(|lang| {
            let dir = build_dir.clone().join(&lang);
            let (git_ref, repo, parsers) = match args.parsers.as_ref().and_then(|p| p.get(&lang)) {
                Some(cli::ParserConfig::Ref(git_ref)) => (
                    Some(git_ref.to_string()),
                    default_repo(&lang),
                    vec![lang.to_string()],
                ),
                Some(cli::ParserConfig::Full {
                    git_ref,
                    from,
                    parsers,
                }) => (
                    Some(git_ref.to_string()),
                    from.as_ref()
                        .map_or_else(|| default_repo(&lang), |f| f.to_string()),
                    parsers.clone().unwrap_or_else(|| vec![lang.to_string()]),
                ),
                _ => (None, default_repo(&lang), vec![lang.to_string()]),
            };
            let pipeline = vec![
                Clone {
                    dir: dir.clone(),
                    git_ref,
                    repo,
                }
                .into(),
                Build {
                    dir: dir.clone(),
                    parsers,
                }
                .into(),
                Copy {
                    from: dir,
                    to: args.out.clone(),
                }
                .into(),
            ];
            Task {
                language: lang.to_string(),
                mailbox: screen.register(lang.to_string(), pipeline.len()),
                pipeline,
            }
        })
        .collect()
}

fn default_repo<L>(lang: L) -> String
where
    L: AsRef<str>,
{
    format!("{}{}", TSP_FROM, lang.as_ref())
}

fn display_dir<D>(dir: D) -> PathBuf
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

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn build(root: PathBuf, lang: String) -> Result<()> {
    debug!("Building {} @ {}", lang, root.display());
    // let get = format!("tree-sitter-{}", lang);
    // let lang_dir = root.join(lang);

    // create_dir_all(root.join("lib")).context("Failed to create lib directory")?;

    // let info = Command::new("sed")
    //     .args(["-n", &format!("-e /^{}|/{{s///;p;q;}}", lang), "ref"])
    //     .output()
    //     .context("Failed to execute sed")?;
    // let info_str = std::str::from_utf8(&info.stdout).context("Failed to parse sed output")?;
    // let url = format!("https://github.com/tree-sitter/{}", get);
    // let (checkout, repo) = info_str
    //     .split_once('|')
    //     .map(|(c, r)| (c.trim(), r.trim()))
    //     .unwrap_or((info_str.trim(), &url));

    // if lang_dir.exists() {
    //     env::set_current_dir(&lang_dir).context("Failed to change directory")?;
    //     Command::new("git")
    //         .args(["reset", "--hard", "HEAD"])
    //         .status()
    //         .context("Failed to reset git repo")?;
    // } else {
    //     create_dir_all(&lang_dir).context("Failed to create language directory")?;
    //     env::set_current_dir(&lang_dir).context("Failed to change directory")?;
    //     Command::new("git")
    //         .args(["init"])
    //         .status()
    //         .context("Failed to init git repo")?;
    //     Command::new("git")
    //         .args(["remote", "add", "origin", repo])
    //         .status()
    //         .context("Failed to add git remote")?;
    // }

    // Command::new("git")
    //     .args(["fetch", "origin", "--depth", "1", checkout])
    //     .status()
    //     .context("Failed to fetch git repo")?;
    // Command::new("git")
    //     .args(["reset", "--hard", "FETCH_HEAD"])
    //     .status()
    //     .context("Failed to reset git repo")?;

    // if lang == "php" {
    //     env::set_current_dir(lang_dir.join("php")).context("Failed to change directory")?;
    //     compile(root, lang_dir.as_path())?;

    //     env::set_current_dir(lang_dir.join("php_only")).context("Failed to change directory")?;
    //     compile(root, lang_dir.as_path())?;
    // } else {
    //     compile(root, lang_dir.as_path())?;
    // }

    Ok(())
}

#[allow(dead_code)]
#[allow(unused_variables)]
fn compile(root: &Path, lang_dir: &Path) -> Result<()> {
    // let where_ = lang_dir.file_name().unwrap().to_str().unwrap();
    // let platform = env::var("PLATFORM").unwrap_or_default();
    // let extout = if build == "s" { "a" } else { "so" };
    // let out = if platform.is_empty() {
    //     root.join(format!("lib/libtree-sitter-{}.{}", where_, extout))
    // } else {
    //     root.join(format!(
    //         "lib/libtree-sitter-{}-{}.{}",
    //         where_, platform, extout
    //     ))
    // };

    // // Regenerate the parser.
    // // Check https://github.com/tree-sitter/tree-sitter/issues/2731
    // tracing::debug!("> {}: regenerating", where_);
    // Command::new("tree-sitter")
    //     .arg("generate")
    //     .status()
    //     .context("Failed to generate parser")?;

    // // clean
    // tracing::debug!("> {}: cleaning", where_);
    // let src_dir = lang_dir.join("src");
    // env::set_current_dir(src_dir).context("Failed to change directory")?;
    // let _ = remove_file("*.o");
    // let _ = remove_file(format!("*.{}", extout));
    // env::set_current_dir(lang_dir).context("Failed to change directory")?;

    // // compile
    // println!("> {}: compiling", where_);
    // Command::new("tree-sitter")
    //     .args(["build", "--output", out.to_str().unwrap()])
    //     .status()
    //     .expect("Failed to build parser");
    Ok(())
}

impl IntoIterator for Task {
    type Item = Pipe;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pipeline.into_iter()
    }
}

impl fmt::Display for Build {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Build {}", display_dir(&self.dir).display())
    }
}

impl fmt::Display for Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repo = match (Url::parse(TSP_FROM), Url::parse(&self.repo)) {
            (Ok(url_from), Ok(url_repo)) => url_from
                .make_relative(&url_repo)
                .unwrap_or(self.repo.to_string()),
            _ => self.repo.to_string(),
        };
        let git_ref = match self.git_ref.as_ref() {
            Some(r) if r.len() == 40 && r.chars().all(|c| c.is_ascii_hexdigit()) => &r[..7],
            Some(r) => r.as_ref(),
            _ => "",
        };
        let disp_dir = display_dir(&self.dir);
        if git_ref.is_empty() {
            write!(f, "Clone {} -> {}", repo, disp_dir.display())
        } else {
            write!(f, "Clone {} @ {} -> {}", repo, git_ref, disp_dir.display())
        }
    }
}

impl fmt::Display for Copy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Copy {} -> {}", self.from.display(), self.to.display())
    }
}

impl fmt::Display for Pipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Pipe::Build(v) => write!(f, "{}", v),
            Pipe::Clone(v) => write!(f, "{}", v),
            Pipe::Copy(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "task::{}", self.language)
    }
}
