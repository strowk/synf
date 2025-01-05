use std::{fs, path::Path};

use eyre::Context;
use inquire::Select;

use crate::{config::{self, Language}, runner, utils};

fn detect_language(folder: &Path) -> config::Language {
    let has_package_json = folder.join("package.json").exists();
    if has_package_json {
        return config::Language::Typescript;
    }
    let has_pyproject_toml = folder.join("pyproject.toml").exists();
    if has_pyproject_toml {
        return config::Language::Python;
    }
    let has_build_gradle = folder.join("build.gradle").exists();
    let has_build_gradle_kts = folder.join("build.gradle.kts").exists();
    if has_build_gradle || has_build_gradle_kts {
        return config::Language::Kotlin;
    }
    let has_go_mod = folder.join("go.mod").exists();
    if has_go_mod {
        return config::Language::Golang;
    }

    config::Language::Typescript
}

pub(crate) fn run(path: Option<String>) -> eyre::Result<()> {
    let folder = if let Some(path) = path {
        path
    } else {
        String::from(".")
    };
    
    let path = Path::new(&folder);
    utils::validate_path(path)?;
    
    let mut language = detect_language(path);
    
    loop {
        let mut options: Vec<&Language> = vec![
           &Language::Typescript,
           &Language::Python,
           &Language::Kotlin,
           &Language::Golang,
        ];

        // sort to put detected language first
        options.sort_by(|a, b| {
            if *a == &language {
                std::cmp::Ordering::Less
            } else if *b == &language {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });

        let ans = Select::new(&format!("Detected language: '{language}', confirm or change:"), options).prompt();
        
        match ans {
            Ok(choice) => {
                language = choice.clone();
                break;
            },
            Err(_) => println!("There was an error, please try again"),
        }
    }

    let mut conf_buf = String::from(
        r####"
# language is used to determine the default paths to watch for changes
# and the default command to run the server.
# Possible values are "typescript", "python", "kotlin" and "golang"
"####,
    );

    match language {
        config::Language::Typescript => {
            conf_buf.push_str(r#"language = "typescript""#);
        }
        config::Language::Python => {
            conf_buf.push_str(r#"language = "python""#);
        }
        config::Language::Kotlin => {
            conf_buf.push_str(r#"language = "kotlin""#);
        }
        config::Language::Golang => {
            conf_buf.push_str(r#"language = "golang""#);
        }
    }

    conf_buf.push_str("\n\n");

    conf_buf.push_str(&format!(
        "[build]
# command and args are used to specify the command to build the server after changes.
# These are the default values for {language}:
"
    ));

    let (build_command, build_args) = runner::Runner::get_build_command(&language);

    conf_buf.push_str(&format!(
        r#"
# command = "{}"
# args = ["{}"]
"#,
        build_command,
        build_args.join("\", \"")
    ));

    conf_buf.push_str(&format!(
        r#"
[run]
# command and args are used to specify the command to run the server
# during development after it has been rebuilt
# These are the default values for {language}:
"#
    ));

    let (run_command, run_args) = runner::Runner::get_run_command(&language);

    conf_buf.push_str(&format!(
        r#"
# command = "{}"
# args = ["{}"]
"#,
        run_command,
        run_args.join("\", \"")
    ));

    conf_buf.push_str(&format!(
        r#"
[watch]
# Watch configurations are used to specify the files and directories to watch for changes
# when hot reloading the server during development

# default_paths are the paths that are watched by default
# and are defined by the language that is being used.
# You can override the default paths by specifying them here.
# These are the paths that are watched by default for {language}:
"#
    ));

    let default_paths = runner::Runner::get_default_watch_paths(&language);

    conf_buf.push_str(&format!(
        r#"
# default_paths = ["{}"]
"#,
        default_paths.join("\", \"")
    ));

    conf_buf.push_str(&format!(
        r#"
# extra_paths are the paths that are watched in addition to the default paths.
# You can use it to add more paths to watch for changes besides the default paths.
# extra_paths = []
"#
    ));

    let path = path.join("synf.toml");

    fs::write(path, conf_buf).context("Failed to write synf.toml file")?;

    Ok(())
}
