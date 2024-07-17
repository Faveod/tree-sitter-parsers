use std::{
    ffi::OsStr,
    fmt::{self},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use derive_more::Display;
use enum_dispatch::enum_dispatch;
use miette::{Context, IntoDiagnostic, Result};
use tokio::sync::mpsc;
use url::Url;

use crate::{args, cli, consts::TSP_FROM, display, git, relative_to_cwd, sh};

pub async fn build_all(
    args: &cli::BuildCommand,
    ts_cli: impl AsRef<Path>,
    screen: &mut display::Screen,
) -> Vec<Result<()>> {
    let languages = if args.languages.as_ref().map_or(true, |l| l.is_empty()) {
        args.parsers
            .as_ref()
            .map_or_else(Vec::new, |m| m.keys().cloned().collect())
    } else {
        args.languages.as_ref().unwrap().clone()
    };
    let build_dir = &args.build_dir;
    let ts_cli_str = Arc::new(ts_cli.as_ref().to_string_lossy().to_string());
    let (tx, mut rx) = mpsc::channel(languages.len());
    for lang in languages {
        let dir = build_dir.clone().join(&lang);
        // TODO: subparsers like php and php_only.
        // TODO: change coordinates for repo, ref, parsers
        let (git_ref, repo, parsers) = parser_coordinates(args, lang);
        let tx = tx.clone();
        let ts_cli_str = ts_cli_str.clone();
        tokio::spawn(async move {
            tx.send(build(repo, git_ref, dir, parsers, ts_cli_str).await)
                .await
                .unwrap();
        });
    }
    drop(tx);
    let mut res = Vec::new();
    while let Some(message) = rx.recv().await {
        res.push(message);
    }
    res
}

async fn build(
    repo: String,
    git_ref: String,
    dir: PathBuf,
    parsers: Vec<String>,
    ts_cli: Arc<String>,
) -> Result<()> {
    sh::exec(&*ts_cli, args!["build"])
        .await
        .context("Building")
        .and(Ok(()))
}

fn default_repo<L>(lang: L) -> String
where
    L: AsRef<str>,
{
    format!("{}{}", TSP_FROM, lang.as_ref())
}

fn parser_coordinates(args: &cli::BuildCommand, lang: String) -> (String, String, Vec<String>) {
    let (git_ref, repo, parsers) = match args.parsers.as_ref().and_then(|p| p.get(&lang)) {
        Some(cli::ParserConfig::Ref(git_ref)) => (
            git_ref.to_string(),
            default_repo(&lang),
            vec![lang.to_string()],
        ),
        Some(cli::ParserConfig::Full {
            git_ref,
            from,
            parsers,
        }) => (
            git_ref.to_string(),
            from.as_ref()
                .map_or_else(|| default_repo(&lang), |f| f.to_string()),
            parsers.clone().unwrap_or_else(|| vec![lang.to_string()]),
        ),
        _ => (
            String::from("HEAD"),
            default_repo(&lang),
            vec![lang.to_string()],
        ),
    };
    (git_ref, repo, parsers)
}

// #[allow(dead_code)]
// #[allow(unused_variables)]
// pub fn build(root: PathBuf, lang: String) -> Result<()> {
//     debug!("Building {} @ {}", lang, root.display());
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

//     Ok(())
// }

// #[allow(dead_code)]
// #[allow(unused_variables)]
// fn compile(root: &Path, lang_dir: &Path) -> Result<()> {
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
//     Ok(())
// }

// impl Run for Build {
//     fn run(&self) -> Result<()> {
//         sh::exec(self.ts_cli.as_ref(), args!["build"])
//             .context("Building")
//             .and(Ok(()))
//     }
// }

// impl Run for Copy {
//     fn run(&self) -> Result<()> {
//         fs::copy(&self.from, &self.to)
//             .into_diagnostic()
//             .context("Copying")
//             .and(Ok(()))
//     }
// }

// impl Run for Clean {
//     fn run(&self) -> Result<()> {
//         if self.dir.exists() && self.dir.is_dir() {
//             fs::read_dir(&self.dir)
//                 .into_diagnostic()?
//                 .filter_map(Result::ok)
//                 .map(|entry| entry.path())
//                 .filter(|path| {
//                     path.is_file()
//                         && path.exists()
//                         && path.extension().map_or(false, |ext| ext == "o")
//                 })
//                 .try_for_each(|path| fs::remove_file(path).into_diagnostic())
//         } else {
//             Ok(())
//         }
//     }
// }

// impl Run for Clone {
//     fn run(&self) -> Result<()> {
//         git::clone_fast(
//             &self.repo,
//             self.git_ref
//                 .as_ref()
//                 .map(|r| r.as_str())
//                 .or(Some("HEAD"))
//                 .unwrap(),
//             self.dir.as_ref(),
//         )
//         .context(format!("Cloning"))
//         .and(Ok(()))
//     }
// }

// impl Run for Compile {
//     fn run(&self) -> Result<()> {
//         std::thread::sleep(std::time::Duration::from_secs(2));
//         Ok(())
//     }
// }

// impl Run for Generate {
//     fn run(&self) -> Result<()> {
//         sh::exec(self.ts_cli.as_ref(), args!["generate"])
//             .context("Generate")
//             .and(Ok(()))
//     }
// }

// impl Run for Task {
//     fn run(&self) -> Result<()> {
//         for p in &self.steps {
//             self.mailbox.send(Some(1), Some(format!("{p}")));
//             p.run()?;
//         }
//         self.mailbox.send(None, Some("Done"));
//         Ok(())
//     }
// }

// impl IntoIterator for Task {
//     type Item = Steps;
//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.steps.into_iter()
//     }
// }

// impl fmt::Display for Build {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Build {}", relative_to_cwd(self.dir.as_ref()).display())
//     }
// }

// impl fmt::Display for Clone {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let repo = match (Url::parse(TSP_FROM), Url::parse(&self.repo)) {
//             (Ok(url_from), Ok(url_repo)) => url_from
//                 .make_relative(&url_repo)
//                 .unwrap_or(self.repo.to_string()),
//             _ => self.repo.to_string(),
//         };
//         let git_ref = match self.git_ref.as_ref() {
//             Some(r) if r.len() == 40 && r.chars().all(|c| c.is_ascii_hexdigit()) => &r[..7],
//             Some(r) => r.as_ref(),
//             _ => "",
//         };
//         let disp_dir = relative_to_cwd(self.dir.as_ref());
//         if git_ref.is_empty() {
//             write!(f, "Clone {} -> {}", repo, disp_dir.display())
//         } else {
//             write!(f, "Clone {} @ {} -> {}", repo, git_ref, disp_dir.display())
//         }
//     }
// }

// impl fmt::Display for Clean {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Clean {}", self.dir.display())
//     }
// }

// impl fmt::Display for Compile {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Compile {}", self.dir)
//     }
// }

// impl fmt::Display for Copy {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Copy {} -> {}", self.from.display(), self.to.display())
//     }
// }

// impl fmt::Display for Generate {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Generate {}", self.dir)
//     }
// }

// impl fmt::Display for Task {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "task::{}", self.language)
//     }
// }
