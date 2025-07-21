use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::search::{Filter, ParamCoverageFilter, ParamTypeFilter};

#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Commands,
}

impl Opts {
    pub fn filter(&self) -> Filter {
        match &self.command {
            Commands::List {
                path: _,
                binary_only,
                public_only,
                json: _,
            } => match (binary_only, public_only) {
                (true, true) => Filter {
                    visibility: Some(rustdoc_types::Visibility::Public),
                    param_type: ParamTypeFilter::BinaryOnly,
                    param_coverage: ParamCoverageFilter::default(),
                },
                (true, false) => Filter {
                    visibility: None,
                    param_type: crate::search::ParamTypeFilter::BinaryOnly,
                    param_coverage: ParamCoverageFilter::default(),
                },
                (false, true) => Filter {
                    visibility: Some(rustdoc_types::Visibility::Public),
                    param_type: ParamTypeFilter::default(),
                    param_coverage: ParamCoverageFilter::default(),
                },
                (false, false) => Filter::default(),
            },
            Commands::Generate {
                inpath: _,
                outpath: _,
            } => todo!(),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    /// Lists viable fuzzing targets
    #[command(arg_required_else_help = true)]
    List {
        #[clap(short, long, action)]
        binary_only: bool,
        #[clap(short, long, action)]
        public_only: bool,
        #[clap(short, long, action)]
        json: bool,
        /// Path to Rust code to search
        path: Option<PathBuf>,
    },
    /// Write fuzzing tests
    Generate {
        /// Path to Rust code to search
        inpath: Option<PathBuf>,
        /// Path to write generated fuzzing tests to
        outpath: Option<PathBuf>,
    },
}
