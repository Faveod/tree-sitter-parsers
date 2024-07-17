use std::path::Path;

use diff::Diff;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use miette::{Context, IntoDiagnostic, Result};
use tracing::debug;

use crate::{
    cli::{BuildCommand, ConfigCommand},
    git,
};

pub fn run<C>(command: &ConfigCommand, config: C) -> Result<()>
where
    C: AsRef<Path>,
{
    match command {
        ConfigCommand::Current => {
            let config: BuildCommand = current(config, None)?;
            println!(
                "{}",
                toml::to_string(&config)
                    .into_diagnostic()
                    .context("Generating default TOML config")?
            );
        }
        ConfigCommand::Default => println!(
            "{}",
            toml::to_string(&BuildCommand::default()).into_diagnostic()?
        ),
    };
    Ok(())
}

pub fn current<C>(config: C, command: Option<&BuildCommand>) -> Result<BuildCommand>
where
    C: AsRef<Path>,
{
    log::debug!("command = {:?}", command);
    let from_default = BuildCommand::default();
    let mut from_file: BuildCommand = Figment::new()
        .merge(Serialized::defaults(from_default.clone()))
        .merge(Toml::file(config.as_ref()))
        .extract()
        .into_diagnostic()?;
    log::debug!("from_file = {:?}", from_file);
    match command {
        Some(from_command) => {
            debug!("Merging cli args + config files");
            let diff = from_default.diff(from_command);
            log::debug!("diff default command = {:?}", diff);
            from_file.apply(&diff);
        }
        None => {
            debug!("Skipping cli args + config file merger.");
        }
    };
    log::debug!("from_both = {:?}", from_file);
    // TODO: read from env vars.
    // Figment is screwing with me, and it's overrinding config coming
    // from Env::prefixed("TSP_").
    // The scary thing is that I might have to write my own config
    // joiner, where I need to track provenance of the config, and also
    // whether it was explicitly set or taken from default … Figment
    // has many features I don't care about.
    Ok(from_file)
}

pub fn print_indent(s: &str, indent: &str) {
    s.lines().for_each(|line| println!("{}{}", indent, line));
}

pub fn show_config(command: &BuildCommand) -> Result<()> {
    match &command.languages {
        Some(langs) => {
            println!("Building the following languages:");
            println!();
            println!(
                "{}",
                String::from_utf8(
                    git::column(langs.join(" "), "  ", 80)
                        .context("Printing requested languages")?
                        .stdout
                )
                .into_diagnostic()
                .context("Converting column-formatted languages to a string for printing")?
            );
        }
        None => {
            println!("Building all languages.");
            println!();
        }
    }
    println!("Running with the following configuration:");
    println!();
    print_indent(
        &toml::to_string(&command)
            .into_diagnostic()
            .context("Showing config")?,
        "  ",
    );
    println!();
    Ok(())
}
