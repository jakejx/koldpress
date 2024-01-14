use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "koldpress")]
pub struct Cli {
    /// Path to the Kobo sqlite db.
    #[arg(short, long, global = true, help = "Path to Kobo DB")]
    pub db_path: Option<std::path::PathBuf>,
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = "Increase logging verbosity",
    )]
    pub verbose: u8,
    /// subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Books(BooksArgs),
    #[command(arg_required_else_help = true)]
    Bookmarks(BookmarkArgs),
}

#[derive(Debug, Args)]
pub struct BooksArgs {
    #[command(subcommand)]
    command: BookCommands,
}

#[derive(Debug, Subcommand)]
pub enum BookCommands {
    List,
}

#[derive(Debug, Args)]
pub struct BookmarkArgs {
    #[command(subcommand)]
    pub command: BookmarkCommands,
}

#[derive(Debug, Subcommand)]
pub enum BookmarkCommands {
    Extract(ExtractArgs),
}

#[derive(Debug, Args)]
pub struct ExtractArgs {
    #[arg(short, long, default_value_t = false)]
    pub all: bool,
    #[arg(value_enum)]
    #[arg(short, long, default_value_t = Format::Json)]
    pub format: Format,
    #[arg(short, long, help = "Output directory")]
    pub output: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Format {
    Json,
    #[value(name = "md")]
    Markdown,
}

impl Format {
    pub fn extension(&self) -> String {
        match self {
            Format::Json => "json".to_string(),
            Format::Markdown => "md".to_string(),
        }
    }
}
