use anyhow::Error;

fn main() -> Result<(), Error> {
    cli::entry()
}

mod args;
mod cli;
mod config;
mod model;
mod op;
mod util;