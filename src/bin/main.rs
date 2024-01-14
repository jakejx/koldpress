use anyhow::{anyhow, Context};
use clap::Parser;
use cli::{ExtractArgs, Format};
use etcetera::{choose_app_strategy, AppStrategy};
use inquire::Select;
use koldpress::kobo::Library;
use koldpress::{config::Config, format};
use sanitize_filename::sanitize;
use std::fs::File;
use std::io::stdout;
use std::path::Path;
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
            cli::BookmarkCommands::Extract(extract) => extract_highlights(library, extract),
        },
    }?;

    Ok(res)
}

fn extract_highlights(library: Library, args: ExtractArgs) -> anyhow::Result<()> {
    let bookmarks = match args.all {
        true => library.get_bookmarks()?,
        false => {
            let books = library.get_books().context("Failed to get books")?;
            let book = Select::new("Book:", books).with_page_size(10).prompt()?;
            info!("Retreving bookmarks for {}", book);
            library.get_bookmarks_for_book(&book)?
        }
    };

    if let Some(ref path) = args.output {
        if !path.is_dir() {
            writeln!(stdout(), "Output must be a directory")?;
            std::process::exit(1);
        }
    }

    for (title, chapters) in bookmarks {
        let io = &mut get_io(title, args.format, args.output.as_deref())?;
        match args.format {
            cli::Format::Json => format::json(io, &chapters)?,
            cli::Format::Markdown => {
                format::markdown(io, &chapters)?;
            }
        }
    }
    Ok(())
}

fn get_io(title: String, format: Format, output: Option<&Path>) -> anyhow::Result<Box<dyn Write>> {
    match output {
        Some(path) => {
            let path = path.join(format!("{}.{}", sanitize(title), format.extension()));
            info!("Opening file: {}", path.display());
            Ok(Box::new(File::create(path)?))
        }
        None => Ok(Box::new(std::io::stdout())),
    }
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
