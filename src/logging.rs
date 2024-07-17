use std::{
    fs::{self, File},
    path::PathBuf,
};

use miette::{Context, IntoDiagnostic, Result};
use tracing_log::AsTrace;

use crate::{cli, consts::TSP_BUILD_DIR};

pub fn start(cli: &cli::Args) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let color = match cli.log_color {
        cli::Color::Auto => atty::is(atty::Stream::Stdout),
        cli::Color::No => true,
        cli::Color::Yes => true,
    };
    let filter = cli.verbose.log_level_filter().as_trace();
    let without_time = std::env::var("TSP_LOG_TIME")
        .map(|v| !matches!(v.to_lowercase().as_str(), "1" | "yes" | "y"))
        .unwrap_or(true);
    let file = prepare_log_file(cli)?;
    init(file, color, filter, without_time)
}

fn init(
    file: File,
    color: bool,
    filter: tracing::level_filters::LevelFilter,
    without_time: bool,
) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let (writer, guard) = tracing_appender::non_blocking(file);
    let builder = tracing_subscriber::fmt()
        .compact()
        .with_ansi(color)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(writer)
        .with_max_level(filter);
    if without_time {
        let subscriber = builder.without_time().finish();
        tracing::subscriber::set_global_default(subscriber)
            .into_diagnostic()
            .context("Setting global tracing without time")?;
    } else {
        let subscriber = builder.finish();
        tracing::subscriber::set_global_default(subscriber)
            .into_diagnostic()
            .context("Setting global tracing with time")?;
    }
    Ok(guard)
}

fn prepare_log_file(cli: &cli::Args) -> Result<File> {
    // Determine log file path.
    let log = cli
        .log
        .as_ref()
        .filter(|l| {
            l.canonicalize()
                .ok()
                .and_then(|p| p.parent().map(|p| p.exists()))
                .unwrap_or_default()
        })
        .cloned()
        .or_else(|| {
            if let Some(cli::Command::Build(b)) = cli.command.as_ref() {
                Some(b.build_dir.clone().join("log"))
            } else {
                None
            }
        })
        .unwrap_or(PathBuf::from(TSP_BUILD_DIR).join("log"));
    // Create its parent dir if needed.
    log.canonicalize()
        .ok()
        .and_then(|path| {
            path.parent().map(|parent| {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .into_diagnostic()
                        .context("Preparing log directory")
                } else {
                    Ok(())
                }
            })
        })
        .transpose()?;
    let file = File::create(&log)
        .into_diagnostic()
        .context("Creating log file")?;
    Ok(file)
}
