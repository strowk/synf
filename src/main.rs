use std::{
    default, env, os,
    path::{self, Path},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use argh::FromArgs;
use config::Watch;
use eyre::Context;
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};

mod config;
mod runner;

#[derive(FromArgs)]
/// MCP Server dev tool for hot reloading
struct Synf {
    #[argh(subcommand)]
    sub: Subcommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommand {
    Dev(Dev),
    // Init(Init),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Run MCP server for development
#[argh(subcommand, name = "dev")]
struct Dev {
    #[argh(positional)]
    path: Option<String>,
}

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    let synf: Synf = argh::from_env();
    match synf.sub {
        Subcommand::Dev(Dev { path }) => {
            let folder = if let Some(path) = path {
                path
            } else {
                String::from(".")
            };

            let path = Path::new(&folder);
            if !path.exists() {
                return Err(eyre::eyre!(format!("Path {:?} does not exist", path)));
            }
            if !path.is_dir() {
                return Err(eyre::eyre!(format!(
                    "Path {:?} expected to be a directory, but was {}",
                    path,
                    if path.is_file() {
                        "a file"
                    } else {
                        "something else"
                    }
                )));
            }
            let cfg =
                config::read_from_toml(path).context("failed to read config from synf.toml")?;

            let therunner = runner::Runner::new(path.to_path_buf(), cfg)?;

            let (tx, rx) = mpsc::channel::<()>();

            ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
                .expect("Error setting Ctrl-C handler");

            println!("Use Ctrl-C to exit.");
            rx.recv().expect("Could not receive from stopping channel.");
            println!("Exiting...");
        }
    }

    // let args: Vec<String> = env::args().collect();

    // let default_path = String::from(".");

    // let folder = args.get(1).unwrap_or(&default_path);
    // let path: &Path = Path::new(folder);
    // if !path.exists() {
    //     return Err(eyre::eyre!(format!("Path {:?} does not exist", path)));
    // }
    // if !path.is_dir() {
    //     return Err(eyre::eyre!(format!(
    //         "Path {:?} expected to be a directory, but was {}",
    //         path,
    //         if path.is_file() {
    //             "a file"
    //         } else {
    //             "something else"
    //         }
    //     )));
    // }

    // let cfg = config::read_from_toml(path).context("failed to read config from synf.toml")?;

    // let therunner = runner::Runner::new(path.to_path_buf(), cfg)?;

    // let (tx, rx) = mpsc::channel::<()>();

    // ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
    //     .expect("Error setting Ctrl-C handler");

    // println!("Use Ctrl-C to exit.");
    // rx.recv().expect("Could not receive from stopping channel.");
    // println!("Exiting...");

    Ok(())
}
