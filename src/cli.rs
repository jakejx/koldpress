use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "koldpress")]
pub(crate) struct Cli {
    /// Path to the Kobo sqlite db.
    #[arg(short, long)]
    pub(crate) db_path: Option<std::path::PathBuf>,
    /// subcommands
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    #[command(arg_required_else_help = true)]
    Books(BooksArgs),
}

#[derive(Debug, Args)]
pub(crate) struct BooksArgs {
    #[command(subcommand)]
    command: BookCommands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum BookCommands {
    List,
    Get(GetBookArgs),
}

#[derive(Args, Debug)]
pub(crate) struct GetBookArgs {
    pub(crate) content_id: Option<String>,
}
