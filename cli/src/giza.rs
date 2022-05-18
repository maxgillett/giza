// use opts::forge::{Dependency, Opts, Subcommands};
pub mod cmd;
mod utils;

use clap::{Parser, Subcommand};
use crate::utils::Cmd;
use cmd::prove::{
    ProveArgs
};

#[derive(Debug, Parser)]
#[clap(name = "giza")]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    Prove(ProveArgs)
}

fn main() {
    let opts = Opts::parse();
    match opts.sub {
        Subcommands::Prove(cmd) => {
            cmd.run();
        }
    }

    // TODO: consider returning Result<T,E> for error codes.
    // Ok(())
}