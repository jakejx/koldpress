use anyhow::anyhow;
use clap::Parser;
use etcetera::{choose_app_strategy, AppStrategy};
use koldpress::config::Config;
use koldpress::kobo::KoboLibrary;
use std::io::Write;
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
    writeln!(stdout, "Reading config from: {}", config_path.display())?;
    let args = cli::Cli::parse();

    let mut config = Config::new(config_path)?;

    if let Some(db_path) = args.db_path {
        config.db_path.get_or_insert(db_path);
    }

    if let Err(err) = config.validate() {
        writeln!(stdout, "Invalid config: {}", err)?;
        std::process::exit(1);
    }

    let library = KoboLibrary::new(config.db_path.ok_or(anyhow!("This should never happen"))?)?;
    let books = library.get_books()?;
    for book in books {
        writeln!(stdout, "Title: {}, Author: {}", book.title, book.author)?;
    }

    Ok(())
}
