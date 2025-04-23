use std::{
    collections::HashMap,
    env,
    io::{stdin, BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, Stdio},
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

use crate::config::{self, Watch};
use crossbeam_channel::{select, unbounded};
use eyre::Context;
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult, RecommendedCache};
use serde::Deserialize;

pub(crate) struct Runner {
    debouncer: Option<notify_debouncer_full::Debouncer<RecommendedWatcher, RecommendedCache>>,
    process: Option<std::process::Child>,
    path: PathBuf,
    build_command: String,
    build_args: Vec<String>,
    run_command: String,
    run_args: Vec<String>,
    language: config::Language,

    client_resource_subscriptions: Arc<Mutex<HashMap<String, String>>>,
    resend_resource_subscriptions: bool,
    client_initialize_req: Arc<Mutex<Option<String>>>,
    process_stopped_sender: Option<crossbeam_channel::Sender<()>>,
    stdin_receiver: Arc<Mutex<crossbeam_channel::Receiver<String>>>,
}

impl Runner {
    fn stop(process: &mut Child) {
        if process.stdin.is_some() {
            eprintln!("Closing running process stdin");
            drop(process.stdin.take());
            // give it some time to close before killing
            thread::sleep(Duration::from_secs(2));
        }

        let should_try_killing: bool;
        match process.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    eprintln!("Process exited successfully");
                } else {
                    eprintln!("Process exited with error: {:?}", status);
                }
                should_try_killing = false;
            }
            Ok(None) => {
                eprintln!("Process is still running");
                should_try_killing = true;
            }
            Err(e) => {
                eprintln!("Failed to check if process is closed: {:?}", e);
                should_try_killing = true;
            }
        }

        if should_try_killing {
            match process.kill() {
                Ok(_) => {
                    eprintln!("Killed running process");
                }
                Err(e) => {
                    eprintln!("Failed to kill running process: {:?}", e);
                }
            }
        }
    }

    pub(crate) fn trigger(&mut self) {
        let build_command = self.build_command.clone();
        let build_args = self.build_args.clone();
        let path = self.path.clone();
        let no_build = self.build_command == "";

        // if windows - wrap in cmd shell, otherwise just run
        if !no_build {
            eprintln!(
                "Running build command: {:?} {:?}",
                build_command, build_args
            );
            let status = std::process::Command::new(build_command)
                .args(build_args)
                .current_dir(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()) // todo: handle stderr to provide logs
                .status();
            match status {
                Ok(status) => {
                    if status.success() {
                        eprintln!("Build succeeded");
                    } else {
                        eprintln!("Build failed");
                    }
                }
                Err(e) => {
                    eprintln!("Error running build command: {:?}", e);
                }
            }
        }

        // Use the configured run command and args from the instance
        // instead of getting the default values again
        let run_command = self.run_command.clone();
        let run_args = self.run_args.clone();

        if let Some(stopped_tx) = &mut self.process_stopped_sender {
            eprintln!("Sending stop to tx");
            stopped_tx.send(()).unwrap();
        }

        if let Some(process) = &mut self.process {
            Self::stop(process);
        }

        eprintln!("Running run command: {:?} {:?}", run_command, run_args);
        let process = std::process::Command::new(run_command)
            .args(run_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .current_dir(self.path.clone())
            .spawn();

        match process {
            Ok(process) => {
                self.process = Some(process);
            }
            Err(e) => {
                eprintln!("Error running run command: {:?}", e);
            }
        }
        eprintln!("Command has started");
        self.run().unwrap();
    }

    pub(crate) fn get_run_command(language: &config::Language) -> (String, Vec<String>) {
        match language {
            config::Language::Typescript => {
                ("node".to_string(), vec!["build/index.js".to_string()])
            }
            config::Language::Python => ("uv".to_string(), vec!["run".to_string()]),
            config::Language::Golang => (
                "go".to_string(),
                vec!["run".to_string(), "main.go".to_string()],
            ),
            config::Language::Kotlin => ("./gradlew".to_string(), vec!["run".to_string()]),
        }
    }

    pub(crate) fn get_build_command(language: &config::Language) -> (String, Vec<String>) {
        match language {
            config::Language::Typescript => (
                "npm".to_string(),
                vec!["run".to_string(), "build".to_string()],
            ),
            config::Language::Python => ("".to_string(), vec![]),
            config::Language::Golang => ("".to_string(), vec![]),
            config::Language::Kotlin => ("".to_string(), vec![]),
        }
    }

    pub(crate) fn get_default_watch_paths(language: &config::Language) -> Vec<String> {
        match language {
            config::Language::Typescript => vec!["src".to_string(), "package.json".to_string()],
            config::Language::Python => vec!["src".to_string(), "pyproject.toml".to_string()],
            config::Language::Golang => vec!["go.mod".to_string()],
            config::Language::Kotlin => vec![
                "src".to_string(),
                "build.gradle.kts".to_string(),
                "gradle.properties".to_string(),
            ],
        }
    }

    pub(crate) fn new(path: PathBuf, cfg: config::Config) -> eyre::Result<Arc<Mutex<Self>>> {
        let (build_command, build_args) = Self::get_build_command(&cfg.language);

        let (build_command, build_args) = if let Some(custom_build_config) = cfg.build {
            let command = custom_build_config.command.unwrap_or(build_command);
            let args = custom_build_config.args.unwrap_or(build_args);
            (command, args)
        } else {
            (build_command, build_args)
        };

        let (run_command, run_args) = Self::get_run_command(&cfg.language);

        let (run_command, run_args) = if let Some(custom_run_config) = cfg.run {
            let command = custom_run_config.command.unwrap_or(run_command);
            let args = custom_run_config.args.unwrap_or(run_args);
            (command, args)
        } else {
            (run_command, run_args)
        };

        let (sender, receiver) = unbounded::<String>();

        thread::spawn(move || {
            for line in stdin().lines() {
                match line {
                    Ok(line) => {
                        sender.send(line).unwrap();
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return;
                    }
                }
            }
        });

        let mut therunner = Runner {
            debouncer: None,
            process: None,
            build_args,
            build_command,
            run_command,
            run_args,
            path: path.clone(),
            language: cfg.language.clone(),

            client_resource_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            resend_resource_subscriptions: cfg.resend_resource_subscriptions.unwrap_or(false),
            client_initialize_req: Arc::new(Mutex::new(None)),
            process_stopped_sender: None,
            stdin_receiver: Arc::new(Mutex::new(receiver)),
        };

        therunner.trigger();

        let runner_arc = Arc::new(Mutex::new(therunner));
        let runner_arc_clone = runner_arc.clone();

        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => events.iter().for_each(|event| {
                    eprintln!("{event:?}");

                    runner_arc_clone.lock().unwrap().trigger();
                }),
                Err(errors) => errors.iter().for_each(|error| eprintln!("{error:?}")),
            },
        )
        .context("failed to create debouncer to watch path")?;

        let default_watch_paths = match &cfg.watch {
            Some(Watch {
                default_paths: Some(configured_default_paths),
                ..
            }) => configured_default_paths.clone(),
            _ => Self::get_default_watch_paths(&cfg.language),
        };

        for watch_path in default_watch_paths {
            let watch_path = path.join(watch_path);
            eprintln!("Watching default path {:?}", watch_path);
            debouncer
                .watch(watch_path.clone(), RecursiveMode::Recursive)
                .with_context(|| format!("failed to watch default path {:?}", watch_path))?;
        }

        if let Some(extra_watch_paths) = cfg.watch.and_then(|w| w.extra_paths) {
            for watch_path in extra_watch_paths {
                eprintln!("Watching extra path {:?}", watch_path);
                let watch_path = path.join(watch_path);
                debouncer
                    .watch(watch_path.clone(), RecursiveMode::Recursive)
                    .with_context(|| format!("failed to watch extra path {:?}", watch_path))?;
            }
        } else {
            eprintln!("No extra watch paths provided");
            if cfg.language == config::Language::Golang {
                eprintln!("Warning: no extra watch paths provided for golang, only watching go.mod, you probably want to add more paths, like internal/, cmd/, etc.");
            }
        }
        runner_arc.lock().unwrap().debouncer = Some(debouncer);
        Ok(runner_arc)
    }

    pub(crate) fn run(&mut self) -> eyre::Result<()> {
        // firstly wait until we have a running process
        while self.process.is_none() {
            // TODO: use channels to wait efficiently
            eprintln!("Waiting for process to start");
            thread::sleep(Duration::from_secs(1));
        }

        let mut process = self.process.take().unwrap();
        let init_req = self.client_initialize_req.clone();

        let mut process_input = process.stdin.take().unwrap();
        let process_out = process.stdout.take().unwrap();

        let (sender, stopped_rx) = unbounded::<()>();
        self.process_stopped_sender = Some(sender);

        let stdin_chan = self.stdin_receiver.clone();

        let resend_resource_subscriptions = self.resend_resource_subscriptions;
        let client_resource_subscriptions = self.client_resource_subscriptions.clone();

        eprintln!("Starting thread to process IO");
        thread::spawn(move || {
            let stdin_chan = stdin_chan.lock().unwrap();

            // phase 1: initialization
            let mut init_req = init_req.lock().unwrap();
            let mut received_client_initialize = false;
            if init_req.is_none() {
                eprintln!("Waiting for input to initialize");
                received_client_initialize = true;
                match stdin_chan.recv() {
                    Ok(line) => {
                        *init_req = Some(line);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return;
                    }
                }
            }

            if let Some(line) = &*init_req {
                process_input
                    .write_all(line.as_bytes())
                    .context("failed to write to process stdin")
                    .unwrap();
                process_input
                    .write_all(b"\n")
                    .context("failed to write to process stdin")
                    .unwrap();
                process_input
                    .flush()
                    .context("failed to flush process stdin")
                    .unwrap();
            }

            let mut process_out = BufReader::new(process_out);
            let mut initialize_response = String::new();
            process_out.read_line(&mut initialize_response).unwrap();

            if received_client_initialize {
                // send initialization response back to client
                print!("{}", initialize_response);
            } else {
                eprintln!("skipping server initialize response");
                // we do not need to send initialize again, as client has
                // already received one from us earlier
                // but we need to imitate client's initialized notification now
                process_input
                    .write(
                        r####"{"method":"notifications/initialized","jsonrpc":"2.0"}
"####
                            .as_bytes(),
                    )
                    .unwrap();

                // we could check here if list of tools actually changed before sending notification
                // , however possibility of this optimisation to misfire probably outweights its value
                //                 process_input
                //                     .write_all(
                //                         r####"{"method":"tools/list","jsonrpc":"2.0","params":{}}
                // "####
                //                             .as_bytes(),
                //                     )
                //                     .unwrap();

                // let mut tools_response = String::new();
                // process_out.read_line(&mut tools_response).unwrap();

                println!(
                    "{}",
                    r####"{"method":"notifications/tools/list_changed","jsonrpc":"2.0"}"####
                );

                println!(
                    "{}",
                    r####"{"method":"notifications/prompts/list_changed","jsonrpc":"2.0"}"####
                );

                println!(
                    "{}",
                    r####"{"method":"notifications/resources/list_changed","jsonrpc":"2.0"}"####
                );

                for (_, subscription) in client_resource_subscriptions.lock().unwrap().iter() {
                    process_input
                        .write_all(subscription.as_bytes())
                        .context("failed to write to process stdin")
                        .unwrap();

                    // skip the responses, since client does not need them
                    let mut dummy_buffer = String::new();
                    process_out.read_line(&mut dummy_buffer).unwrap();
                }
            }

            // phase 2: proxying
            thread::spawn(move || {
                eprintln!("started stdout processing");
                for line in process_out.lines() {
                    match line {
                        Ok(line) => {
                            println!("{}", line);
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            eprintln!("ended stdout processing");
                            return;
                        }
                    }
                }
                eprintln!("exit ended stdout processing");
            });

            loop {
                select! {
                    recv(stopped_rx) -> _ => {
                        eprintln!("exiting input processing loop by rx");
                        break;
                    }
                    recv(stdin_chan) -> line => {
                        match line {
                            Ok(line) => {
                                if resend_resource_subscriptions {
                                    if line.contains("resources/subscribe") || line.contains("resources/unsubscribe") {
                                        let partial: serde_json::Result<PartialJsonRequest> = serde_json::from_str(&line);
                                        if let Ok(partial) = partial {
                                            let mut client_resource_subscriptions = client_resource_subscriptions.lock().unwrap();
                                            if partial.method == "resources/subscribe" {
                                                client_resource_subscriptions.insert(partial.params.uri.clone(), line.clone());
                                            } else if partial.method == "resources/unsubscribe" {
                                                client_resource_subscriptions.remove(&partial.params.uri);
                                            }
                                        }
                                    }
                                }
                                process_input
                                .write_all(line.as_bytes())
                                .context("failed to write to process stdin")
                                .unwrap();
                                process_input
                                    .write_all(b"\n")
                                    .context("failed to write to process stdin")
                                    .unwrap();
                                process_input
                                    .flush()
                                    .context("failed to flush process stdin")
                                    .unwrap();
                            }
                            Err(e)=>{
                                eprintln!("Failed to get from chan {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            drop(process_input);

            // give it some time to close before killing
            thread::sleep(Duration::from_secs(2));

            Self::stop(&mut process);

            eprintln!("Finished proxying");
        });

        Ok(())
    }

    // pub(crate) fn stop(self) {
    //     if let Some(debouncer) = self.debouncer {
    //         debouncer.stop();
    //     }
    // }
}

#[derive(Deserialize)]
struct PartialJsonRequest {
    pub method: String,
    pub params: SubscribeParams,
}

#[derive(Deserialize)]
struct SubscribeParams {
    pub uri: String,
}
