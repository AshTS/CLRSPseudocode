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
    Parse {
        file: PathBuf
    },
    Execute {
        file: PathBuf
    },
    Compile {
        file: PathBuf
    },
    VMRun {
        file: PathBuf,
        /// Supress the visualization
        #[clap(short, long, action)]
        supress: bool,

        /// Don't wait for input
        #[clap(short, long, action)]
        no_wait: bool,

        /// Shows instructions as they are executed
        #[clap(short, long, action)]
        instructions: bool
    },
}