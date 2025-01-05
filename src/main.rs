use std::{path::Path, sync::mpsc};

use argh::FromArgs;
use eyre::Context;

mod config;
mod init;
mod runner;
mod utils;

#[derive(FromArgs)]
/// Tool for MCP Server hot reloading 
struct Synf {
    #[argh(subcommand)]
    sub: Subcommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommand {
    Dev(Dev),
    Init(Init),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Run MCP server for development with hot reloading
#[argh(subcommand, name = "dev")]
struct Dev {
    #[argh(positional)]
    path: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize project by preparing synf.toml file
#[argh(subcommand, name = "init")]
struct Init {
    #[argh(positional)]
    path: Option<String>,
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let synf: Synf = argh::from_env();
    match synf.sub {
        Subcommand::Init(Init { path }) => {
            return init::run(path);
        }
        Subcommand::Dev(Dev { path }) => {
            let folder = if let Some(path) = path {
                path
            } else {
                String::from(".")
            };

            let path = Path::new(&folder);
            utils::validate_path(path)?;
            let cfg =
                config::read_from_toml(path).context("failed to read config from synf.toml")?;

            runner::Runner::new(path.to_path_buf(), cfg)?;

            let (tx, rx) = mpsc::channel::<()>();

            ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
                .expect("Error setting Ctrl-C handler");

            eprintln!("Use Ctrl-C to exit.");
            rx.recv().expect("Could not receive from stopping channel.");
        }
    }

    Ok(())
}
