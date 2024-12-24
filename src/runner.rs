use std::{
    default, env, os,
    path::{self, Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::config::{self, Watch};
use eyre::Context;
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult, FileIdMap};

pub(crate) struct Runner {
    debouncer: Option<notify_debouncer_full::Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
    process: Option<std::process::Child>,
    path: PathBuf,
    build_command: String,
    build_args: Vec<String>,
    language: config::Language,
}

impl Runner {
    pub(crate) fn trigger(&mut self) {
        let build_command = self.build_command.clone();
        let build_args = self.build_args.clone();
        let path = self.path.clone();
        let no_build = self.build_command == "";

        // if windows - wrap in cmd shell, otherwise just run
        if !no_build {
            let (build_command, build_args) = if env::consts::OS == "windows" {
                {
                    let mut powershell_args = vec![build_command];
                    powershell_args.append(&mut build_args.clone());
                    (
                        "powershell.exe".to_string(),
                        // vec!["-Command".to_string(), command_in_shell],
                        // build_args,
                        powershell_args,
                    )
                }
            } else {
                (build_command, build_args)
            };

            println!(
                "Running build command: {:?} {:?}",
                build_command, build_args
            );
            let status = std::process::Command::new(build_command)
                .args(build_args)
                .current_dir(path)
                .status();
            match status {
                Ok(status) => {
                    if status.success() {
                        println!("Build succeeded");
                    } else {
                        println!("Build failed");
                    }
                }
                Err(e) => {
                    println!("Error running build command: {:?}", e);
                }
            }
        }

        let (run_command, run_args) = match self.language {
            config::Language::Typescript => ("node", vec!["build/index.js"]),
            config::Language::Python => ("uv", vec!["run"]),
            config::Language::Golang => ("go", vec!["run"]),
        };

        if let Some(process) = &mut self.process {
            if process.stdin.is_some() {
                println!("Closing running process stdin");
                drop(process.stdin.take());
                // give it some time to close before killing
                thread::sleep(Duration::from_secs(2));
            }

            let should_try_killing: bool;
            match process.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        println!("Process exited successfully");
                    } else {
                        println!("Process exited with error: {:?}", status);
                    }
                    should_try_killing = false;
                }
                Ok(None) => {
                    println!("Process is still running");
                    should_try_killing = true;
                }
                Err(e) => {
                    println!("Failed to check if process is closed: {:?}", e);
                    should_try_killing = true;
                }
            }

            if should_try_killing {
                match process.kill() {
                    Ok(_) => {
                        println!("Killed running process");
                    }
                    Err(e) => {
                        println!("Failed to kill running process: {:?}", e);
                    }
                }
            }
        }

        let run_args = run_args.clone();
        let (run_command, run_args) = if env::consts::OS == "windows" {
            let mut powershell_args = vec![run_command];
            powershell_args.append(&mut run_args.clone());
            ("powershell.exe", powershell_args)
        } else {
            (run_command, run_args)
        };

        println!("Running run command: {:?} {:?}", run_command, run_args);
        let process = std::process::Command::new(run_command)
            .args(run_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(self.path.clone())
            .spawn();

        match process {
            Ok(process) => {
                self.process = Some(process);
            }
            Err(e) => {
                println!("Error running run command: {:?}", e);
            }
        }
        println!("Command has started");
    }

    pub(crate) fn new(path: PathBuf, cfg: config::Config) -> eyre::Result<Arc<Mutex<Self>>> {
        let (build_command, build_args) = match cfg.language {
            config::Language::Typescript => (
                "npm".to_string(),
                vec!["run".to_string(), "build".to_string()],
            ),
            // these should do both build and run in run command
            config::Language::Python => ("".to_string(), vec![]),
            config::Language::Golang => ("".to_string(), vec![]),
        };

        let mut therunner = Runner {
            debouncer: None,
            process: None,
            build_args,
            build_command,
            path: path.clone(),
            language: cfg.language.clone(),
        };

        therunner.trigger();

        let runner_arc = Arc::new(Mutex::new(therunner));
        let runner_arc_clone = runner_arc.clone();

        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => events.iter().for_each(|event| {
                    println!("{event:?}");

                    runner_arc_clone.lock().unwrap().trigger();
                }),
                Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
            },
        )
        .context("failed to create debouncer to watch path")?;

        let default_watch_paths = match &cfg.watch {
            Some(Watch {
                default_watch_paths: Some(configured_default_paths),
                ..
            }) => configured_default_paths.clone(),
            _ => {
                if cfg.language == config::Language::Typescript {
                    vec!["src".to_string(), "package.json".to_string()]
                } else if cfg.language == config::Language::Python {
                    vec!["src".to_string(), "pyproject.toml".to_string()]
                } else if cfg.language == config::Language::Golang {
                    vec!["go.mod".to_string()]
                } else {
                    return Err(eyre::eyre!(format!(
                        "Unsupported language {:?}",
                        &cfg.language
                    )));
                }
            }
        };

        for watch_path in default_watch_paths {
            let watch_path = path.join(watch_path);
            println!("Watching default path {:?}", watch_path);
            debouncer
                .watch(watch_path.clone(), RecursiveMode::Recursive)
                .with_context(|| format!("failed to watch default path {:?}", watch_path))?;
        }

        if let Some(extra_watch_paths) = cfg.watch.and_then(|w| w.extra_watch_paths) {
            for watch_path in extra_watch_paths {
                println!("Watching extra path {:?}", watch_path);
                let watch_path = path.join(watch_path);
                debouncer
                    .watch(watch_path.clone(), RecursiveMode::Recursive)
                    .with_context(|| format!("failed to watch extra path {:?}", watch_path))?;
            }
        } else {
            println!("No extra watch paths provided");
            if cfg.language == config::Language::Golang {
                println!("Warning: no extra watch paths provided for golang, only watching go.mod, you probably want to add more paths, like internal/, cmd/, etc.");
            }
        }
        runner_arc.lock().unwrap().debouncer = Some(debouncer);
        Ok(runner_arc)
    }

    pub(crate) fn run(&self) -> eyre::Result<()> {
        println!("Hello, world!");
        Ok(())
    }

    // pub(crate) fn stop(self) {
    //     if let Some(debouncer) = self.debouncer {
    //         debouncer.stop();
    //     }
    // }
}
