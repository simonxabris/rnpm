use clap::{App, Arg};
use dialoguer;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process};
use std::env::current_dir;
use std::fs::read_to_string;
use std::process::{Command, Stdio};

fn main() {
    let args = App::new("rnpm")
        .about("Find the script you're looking for.")
        .arg(Arg::with_name("script")
            .takes_value(false)
            .required(false)
            .help("The string you're searching for")
        )
        .arg(
            Arg::with_name("manager")
                .short("m")
                .long("manager")
                .value_name("PACKAGE MANAGER")
                .takes_value(true)
                .possible_values(&["npm", "yarn", "pnpm"])
                .required(false)
        )
        .arg(
            Arg::with_name("run-exact")
                .short("r")
                .long("run-exact")
                .takes_value(false)
                .required(false)
                .help("If set, immediately run the script that matches exactly.")
        )
        .get_matches();

    let has_yarn_lock = std::fs::File::open("yarn.lock").is_ok();
    let has_pnpm_lock = std::fs::File::open("pnpm-lock.yaml").is_ok();

    let possible_search_arg = args.value_of("script");
    let package_manager = if let Some(manager) = args.value_of("manager") {
        manager
    } else {
        match (has_yarn_lock, has_pnpm_lock) {
            (true, false) => "yarn",
            (false, true) => "pnpm",
            _ => "npm"
        }
    };
    let run_exact_flag = args.is_present("run-exact");

    println!("run exact flag: {:?}", run_exact_flag);

    let mut cwd = current_dir().expect("Couldn't access current directory");
    cwd.push("package.json");

    let package_json_contents = read_to_string(cwd).expect("Could not find package.json file.");

    let package_json: PackageJson =
        serde_json::from_str(&package_json_contents[..]).expect("package.json is not valid JSON.");

    let scripts: Vec<&String> = match possible_search_arg {
        Some(arg) => package_json.scripts.keys().filter(|script| {
            if script == &arg && run_exact_flag {
                execute_command(package_manager, arg);
            }

            script.contains(arg)
        }).collect(),
        None => package_json.scripts.keys().collect()
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

    execute_command(package_manager, scripts[selected_script_index]);
}

fn execute_command(package_manager: &str, command: &str) -> ! {
    let mut child = Command::new(package_manager)
        .arg("run")
        .arg(command)
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn child process.");

    child.wait().expect("Child Process failed");

    process::exit(0);
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    scripts: HashMap<String, String>,
}
