use dialoguer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::args;
use std::env::current_dir;
use std::fs::read_to_string;
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = args().collect();
    let possible_search_arg = args.get(1);

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

    let mut child = Command::new("npm")
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
