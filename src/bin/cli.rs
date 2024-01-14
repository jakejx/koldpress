use std::{path::PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "koldpress")]
pub(crate) struct Cli {
    /// Path to the Kobo sqlite db.
    #[arg(short, long, global = true, help = "Path to Kobo DB")]
    pub(crate) db_path: Option<std::path::PathBuf>,
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = "Increase logging verbosity",
    )]
    pub(crate) verbose: u8,
    /// subcommands
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    #[command(arg_required_else_help = true)]
    Books(BooksArgs),
    #[command(arg_required_else_help = true)]
    Bookmarks(BookmarkArgs),
}

#[derive(Debug, Args)]
pub(crate) struct BooksArgs {
    #[command(subcommand)]
    command: BookCommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum BookCommands {
    List,
}

#[derive(Debug, Args)]
pub(crate) struct BookmarkArgs {
    #[command(subcommand)]
    pub(crate) command: BookmarkCommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum BookmarkCommands {
    Extract(ExtractArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ExtractArgs {
    #[arg(short, long, default_value_t = false)]
    pub(crate) all: bool,
    #[arg(value_enum)]
    #[arg(short, long, default_value_t = Format::Json)]
    pub(crate) format: Format,
    #[arg(short, long, help = "Output directory")]
    pub(crate) output: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum Format {
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
