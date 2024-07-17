use std::{collections::BTreeMap, fmt, path::PathBuf};

use clap_verbosity_flag::{InfoLevel, Verbosity};
use diff::Diff;
use serde::{Deserialize, Serialize};

use crate::consts::*;

/// Command-line arguments.
#[derive(Clone, Debug, Deserialize, clap::Parser, Serialize)]
#[command(author, version, about, long_about = None, allow_external_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to the config file (TOML).
    #[arg(short, long, default_value = "parsers.toml")]
    pub config: PathBuf,

    /// Path to the logging file. If unspecified, it will go to build_dir/log.
    #[arg(short, long)]
    pub log: Option<PathBuf>,

    /// Whether to emit colored logs.
    #[arg(long, value_enum, default_value_t = Color::Auto)]
    pub log_color: Color,

    /// Verbosity level: -v, -vv, -vvv, or -q, -qq, -qqq
    /// clap_verbosity_flag, as of now, refuses to add a serialization feature, so this will not be part of the config file.
    #[serde(skip_serializing, skip_deserializing)]
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
}

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, Serialize)]
pub enum Color {
    Auto,
    No,
    Yes,
}

#[derive(clap::Subcommand, Clone, Debug, Deserialize, Serialize)]
pub enum Command {
    /// Build one or many parsers.
    Build(BuildCommand),

    /// Configuration helpers.
    #[serde(skip_serializing, skip_deserializing)]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(clap::Args, Clone, Debug, Deserialize, Diff, PartialEq, Eq, Serialize)]
#[diff(attr(
    #[derive(Debug, PartialEq)]
))]
pub struct BuildCommand {
    /// Parsers to compile as key=value pairs.
    /// values can be either:
    ///   1. a simple value denoting the ref of the parser from the default remote.
    ///   2. of the format `ref:remote-ref,from:remote-url`.
    ///     `ref` and `from` are both optional, and will use defualts.
    #[serde(skip_serializing, skip_deserializing)]
    #[arg(verbatim_doc_comment)]
    pub languages: Option<Vec<String>>,

    /// Configured Parsers.
    #[clap(skip)]
    pub parsers: Option<BTreeMap<String, ParserConfig>>,

    /// Build Directory.
    #[serde(rename = "build-dir")]
    #[arg(short, long, default_value = TSP_BUILD_DIR)]
    pub build_dir: PathBuf,

    /// Whether parsers should be statically linked to tree-sitter.
    #[serde(rename = "static")]
    #[arg(short = 's', long = "static", default_value_t = TSP_STATIC)]
    pub build_static: bool,

    /// Number of parallel jobs; defaults to the number of available CPUs.
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub jobs: usize,

    /// Clears the build_dir and starts a fresh build.
    #[arg(short, long, default_value_t = TSP_FRESH)]
    pub fresh: bool,

    /// Output Directory.
    #[arg(short, long, default_value = TSP_OUT)]
    pub out: PathBuf,

    /// Show Config.
    #[serde(rename = "show-config")]
    #[clap(alias = "show-config")]
    #[arg(long, default_value_t = TSP_SHOW_CONFIG)]
    pub show_config: bool,

    #[serde(rename = "tree-sitter")]
    #[command(flatten)]
    pub tree_sitter: TreeSitter,
}

impl Default for BuildCommand {
    fn default() -> Self {
        Self {
            languages: None,
            parsers: None,
            build_dir: PathBuf::from(TSP_BUILD_DIR),
            build_static: TSP_STATIC,
            fresh: TSP_FRESH,
            jobs: num_cpus::get(),
            out: PathBuf::from(TSP_OUT),
            show_config: TSP_SHOW_CONFIG,
            tree_sitter: TreeSitter::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Diff, Serialize, PartialEq, Eq)]
#[diff(attr(
    #[derive(Debug, PartialEq)]
))]
#[serde(untagged)]
pub enum ParserConfig {
    Full {
        #[serde(rename = "ref")]
        git_ref: String,
        from: Option<String>,
        parsers: Option<Vec<String>>,
    },
    Ref(String),
}

#[derive(clap::Args, Clone, Debug, Diff, Deserialize, PartialEq, Eq, Serialize)]
#[diff(attr(
    #[derive(Debug, PartialEq)]
))]
pub struct TreeSitter {
    /// Tree-sitter version.
    #[arg(short = 'V', long = "tree-sitter-version", default_value = TREE_SITTER_VERSION)]
    pub version: String,

    /// Tree-sitter repo.
    #[arg(short = 'R', long = "tree-sitter-repo", default_value = TREE_SITTER_REPO)]
    pub repo: String,

    // Tree-sitter plarform to build.
    #[arg(short = 'P', long = "tree-sitter-platform", default_value = TREE_SITTER_PLATFORM)]
    pub platform: String,
}

impl Default for TreeSitter {
    fn default() -> Self {
        Self {
            version: TREE_SITTER_VERSION.to_string(),
            repo: TREE_SITTER_REPO.to_string(),
            platform: TREE_SITTER_PLATFORM.to_string(),
        }
    }
}

#[derive(clap::Subcommand, Clone, Debug, Default)]
pub enum ConfigCommand {
    #[default]
    Current,
    Default,
}

impl fmt::Display for ConfigCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", format!("{:?}", self).to_lowercase())
    }
}
