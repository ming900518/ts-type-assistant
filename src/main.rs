use std::{error::Error, time::Instant};

use clap::Parser;
use cli::{Arg, Command};
use parser::{parse_all, PARSED};

mod cli;
mod parser;
mod types;

fn main() -> Result<(), Box<dyn Error>> {
    match Arg::parse().command {
        Command::Lsp => lsp(),
        Command::Manually(options) => {
            let now = Instant::now();
            parse_all(&options.get_all_input_files()?);
            PARSED.get().map_or_else(
                || {
                    println!("No information available.");
                },
                |parsed| {
                    println!("Finished in {} ms.", now.elapsed().as_millis());
                    println!("{parsed:#?}");
                },
            );
        }
    };

    Ok(())
}

fn lsp() {
    todo!()
}
