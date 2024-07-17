use std::sync::mpsc;
use std::{collections::HashSet, fs};

use miette::{miette, Context, IntoDiagnostic, Result};
use tracing::{debug, error};

use crate::parser::Executable;
use crate::{cli, config, parser, progress, tree_sitter};

pub fn run(command: &cli::BuildCommand) -> Result<()> {
    if command.show_config {
        config::show_config(command)?;
    }
    if command.fresh && command.build_dir.exists() {
        fs::remove_dir_all(&command.build_dir)
            .into_diagnostic()
            .context(format!(
                "Removing the build_dir {} for a fresh build",
                &command.build_dir.display()
            ))?;
    }
    fs::create_dir_all(&command.build_dir)
        .into_diagnostic()
        .context("Creating the build dir")?;
    tree_sitter::build(command).context("Building tree-sitter")?;
    let languages = requested_languages(command);
    debug!("Compiling {:?}", &languages);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(command.jobs)
        .build()
        .unwrap();
    let results = build_parallel(command, pool);
    for res in results {
        match res {
            Ok(_) => println!("Done"),
            Err(e) => println!("Failed:\n{}", e),
        }
    }
    Ok(())
}

fn build_parallel(command: &cli::BuildCommand, pool: rayon::ThreadPool) -> Vec<Result<()>> {
    let mut screen = progress::Screen::new();
    let (tx, rx) = mpsc::channel();
    let tasks = parser::tasks(command, &mut screen);
    for task in tasks {
        let tx = tx.clone();
        pool.spawn(move || {
            if let Err(err) = tx.send(task.run()) {
                error!("Sending {} results failed", task);
                error!("{}", err);
            }
        });
    }
    drop(tx);
    rx.into_iter().collect::<Vec<_>>()
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
