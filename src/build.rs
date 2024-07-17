use std::{collections::HashSet, fs};

use miette::{bail, Context, IntoDiagnostic, Result};
use tracing::{debug, error};

use crate::{cli, config, display, parser, tree_sitter};

pub fn run(command: &cli::BuildCommand) -> Result<()> {
    if command.show_config {
        config::show_config(command)?;
    }
    let mut screen = display::Screen::new();
    clear(command, &mut screen)?;
    ignite(command, &mut screen)?;
    Ok(())
}

fn clear(command: &cli::BuildCommand, screen: &mut display::Screen) -> Result<(), miette::Error> {
    if command.fresh && command.build_dir.exists() {
        let handle = screen.register("Fresh Build", 1);
        let disp = &command.build_dir.display();
        fs::remove_dir_all(&command.build_dir)
            .into_diagnostic()
            .context(format!("Removing the build_dir {} for a fresh build", disp))?;
        handle.send(Some(1), Some(format!("Cleaned {}", disp)))
    }
    fs::create_dir_all(&command.build_dir)
        .into_diagnostic()
        .context("Creating the build dir")?;
    Ok(())
}

fn ignite(command: &cli::BuildCommand, screen: &mut display::Screen) -> Result<(), miette::Error> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(command.jobs)
        .build();
    if let Err(err) = rt {
        eprintln!("Failed to initialize tokio.");
        eprintln!("{err}");
        bail!("Failed to spawn the tokio runtime");
    }
    let rt = rt.unwrap();
    let _guard = rt.enter();
    let ts_cli = rt.block_on(async {
        tree_sitter::build(&command, screen)
            .await
            .context("Building tree-sitter")
    })?;
    let languages = requested_languages(command);
    debug!("Compiling {:?}", &languages);
    rt.block_on(async {
        let results = parser::build_all(command, ts_cli, screen).await;
        for res in results {
            match res {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed:\n{}", e)
                }
            }
        }
    });
    Ok(())
}

fn requested_languages(command: &cli::BuildCommand) -> Vec<String> {
    let mut languages = command
        .languages
        .clone()
        .filter(|arr| !arr.is_empty())
        .or_else(|| {
            command
                .parsers
                .as_ref()
                .map(|map| map.keys().cloned().collect())
        })
        .unwrap_or_default();
    if !languages.is_empty() {
        languages = languages
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
    }
    languages
}
