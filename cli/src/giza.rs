pub mod cmd;
mod utils;

use crate::utils::Cmd;
use clap::{Parser, Subcommand};
use cmd::prove::ProveArgs;
use cmd::verify::VerifyArgs;

#[derive(Debug, Parser)]
#[clap(name = "giza")]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    Prove(ProveArgs),
    Verify(VerifyArgs),
}

fn main() {
    let opts = Opts::parse();
    match opts.sub {
        Subcommands::Prove(cmd) => {
            cmd.run().unwrap();
        }
        Subcommands::Verify(cmd) => {
            cmd.run().unwrap();
        }
    }

    // TODO: consider returning Result<T,E> for error codes.
    // Ok(())
}
