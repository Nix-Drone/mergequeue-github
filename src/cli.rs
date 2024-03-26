use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "Trunk Technologies Inc.")]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommands>,

    #[clap(long = "gh-token")]
    #[arg(default_value_t = String::from(""))]
    pub gh_token: String,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Generate default configuration content for generator
    Genconfig,
}
