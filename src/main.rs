use clap::Parser;
use miette::{Context, Result};
use tracing::{error, info};

use tsp::{build, cli, config, logging};

fn main() -> Result<()> {
    set_panic_hook();
    let cli = cli::Args::parse();
    let _guard = logging::start(&cli)?;
    info!("Starting");
    run(&cli)?;
    info!("Done");
    Ok(())
}

fn run(cli: &cli::Args) -> Result<()> {
    match &cli.command {
        Some(cli::Command::Build(command)) => build::run(
            &config::current(&cli.config, Some(command)).context("Figuring out current config")?,
        )
        .context("Running build"),
        Some(cli::Command::Config { command }) => {
            config::run(command, &cli.config).context(format!("Running config {}", command))
        }
        _ => {
            build::run(&config::current(&cli.config, None).context("Figuring out current config")?)
                .context("Running default build")
        }
    }
}

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(move |info| {
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};
            let meta = Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .authors(env!("CARGO_PKG_AUTHORS").replace(':', ", "))
                .homepage(env!("CARGO_PKG_HOMEPAGE"));

            let file_path = handle_dump(&meta, info);
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }
        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(info);
        }
        error!("{}", info);
        std::process::exit(1);
    }));
}
