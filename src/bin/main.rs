use anyhow::{anyhow, Context};
use clap::Parser;
use cli::ExtractArgs;
use etcetera::{choose_app_strategy, AppStrategy};
use inquire::Select;
use koldpress::config::Config;
use koldpress::kobo::Library;
use std::{
    io::{self, Write},
    str::FromStr,
};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*};

mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let strategy = choose_app_strategy(etcetera::AppStrategyArgs {
        app_name: "koldpress".to_string(),
        top_level_domain: "".to_string(),
        author: "jakejx".to_string(),
    })
    .unwrap();

    let config_path = strategy.in_config_dir("config");
    info!("Reading config from: {}", config_path.display());
    let args = cli::Cli::parse();
    init_tracing(args.verbose);

    let mut config = Config::new(config_path)?;

    if let Some(db_path) = args.db_path {
        config.db_path.get_or_insert(db_path);
    }

    if let Err(err) = config.validate() {
        writeln!(stdout, "Invalid config: {}", err)?;
        std::process::exit(1);
    }

    let library = Library::new(config.db_path.ok_or(anyhow!("This should never happen"))?)?;

    let res = match args.command {
        cli::Commands::Books(_) => todo!(),
        cli::Commands::Bookmarks(bookmarks) => match bookmarks.command {
            cli::BookmarkCommands::Extract(extract) => {
                extract_highlights(library, extract, Box::new(stdout))
            }
        },
    }?;

    Ok(res)
}

fn extract_highlights(
    library: Library,
    args: ExtractArgs,
    mut io: Box<dyn Write>,
) -> anyhow::Result<()> {
    let bookmarks = match args.all {
        true => library.get_bookmarks()?,
        false => {
            let books = library.get_books().context("Failed to get books")?;
            let book = Select::new("Book:", books).with_page_size(10).prompt()?;
            info!("Retreving bookmarks for {}", book);
            library.get_bookmarks_for_book(&book)?
        }
    };

    match args.format {
        cli::Format::Json => write!(io, "{}", serde_json::to_string_pretty(&bookmarks)?)?,
        cli::Format::Markdown => todo!(),
    }
    Ok(())
}

fn init_tracing(verbosity: u8) {
    let filter =
        LevelFilter::from_str(&verbosity.to_string()).expect("verbosity level not a number");
    let format = fmt::layer().compact().with_writer(io::stderr);
    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();
}
