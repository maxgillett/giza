use crate::{
    utils::{Cmd},
};
use clap::{Parser, ValueHint};
use std::{path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProveArgs {
    #[clap(
        help = "A path to the execution trace.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub trace: PathBuf,

    #[clap(
        help = "A path to write the STARK proof.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub output: PathBuf,
}

pub struct ProveOutput {}

impl Cmd for ProveArgs {
    type Output = ProveOutput;

    fn run(self) -> Self::Output {
        ProveOutput {}
    }
}