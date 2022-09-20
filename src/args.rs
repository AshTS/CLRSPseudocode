use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Arguments {
    /// Subcommand
    #[clap(subcommand)]
    pub sub_command: SubCommand
}

#[derive(clap::Subcommand, Debug)]
#[non_exhaustive]
pub enum SubCommand {
    Tokenize {
        file: PathBuf
    },
    Parser {
        file: PathBuf
    },
    Execute {
        file: PathBuf
    },
    Compile {
        file: PathBuf
    },
    VMRun {
        file: PathBuf
    },
}