use clap::{App, Arg};
use dialoguer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_to_string;
use std::process::{Command, Stdio};

fn main() {
    let args = App::new("rnpm")
        .about("Find the script you're looking for.")
        .arg(Arg::with_name("script").takes_value(false).required(false))
        .arg(
            Arg::with_name("manager")
                .short("m")
                .long("manager")
                .value_name("PACKAGE MANAGER")
                .takes_value(true)
                .possible_values(&["npm", "yarn", "pnpm"])
                .required(false)
                .default_value("npm"),
        )
        .get_matches();

    let possible_search_arg = args.value_of("script");
    // safe because the manager arg has a default value.
    let possible_package_manager = args.value_of("manager").unwrap();

    let mut cwd = current_dir().expect("Couldn't access current directory");
    cwd.push("package.json");

    let package_json_contents = read_to_string(cwd).expect("Could not find package.json file.");

    let package_json: PackageJson =
        serde_json::from_str(&package_json_contents[..]).expect("package.json is not valid JSON.");

    let scripts: Vec<&String> = match possible_search_arg {
        Some(arg) => package_json
            .scripts
            .keys()
            .filter(|script| script.contains(arg))
            .collect(),
        None => package_json.scripts.keys().collect(),
    };

    if scripts.len() == 0 {
        println!("Could not find any scripts.");
        std::process::exit(0);
    }

    let selector = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&scripts)
        .default(0)
        .interact_on_opt(&dialoguer::console::Term::stdout())
        .expect("Failed to display options");

    let selected_script_index = selector.expect("No value was selected.");

    let mut child = Command::new(possible_package_manager)
        .arg("run")
        .arg(scripts[selected_script_index])
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn child process.");

    child.wait().expect("Child Process failed");
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    scripts: HashMap<String, String>,
}
