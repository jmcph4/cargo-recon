use std::path::PathBuf;

use clap::Parser;
use cli::Opts;
use search::search_file;

use crate::cli::Commands;

pub mod cli;
pub mod rustdoc;
pub mod search;

fn main() -> eyre::Result<()> {
    pretty_env_logger::init_timed();
    let opts = Opts::parse();

    match opts.command {
        Commands::List {
            ref path,
            binary_only: _,
            public_only: _,
            json,
        } => {
            let path = match path {
                Some(p) => p,
                None => {
                    let mut p = PathBuf::new();
                    p.push(".");
                    &p.clone()
                }
            };

            let targets = search_file(path, Some(opts.filter()), !opts.quiet)?;

            if json {
                println!("{}", serde_json::to_string(&targets).unwrap());
            } else {
                targets.iter().for_each(|target| println!("{target}"));
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
}
