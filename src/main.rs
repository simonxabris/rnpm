use dialoguer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_to_string;
use std::process::{Command, Stdio};

fn main() {
    let mut cwd = current_dir().expect("Couldn't access current directory");
    cwd.push("package.json");

    let package_json_contents = read_to_string(cwd).expect("Could not find package.json file.");

    let package_json: PackageJson =
        serde_json::from_str(&package_json_contents[..]).expect("package.json is not valid JSON.");

    let scripts: Vec<&String> = package_json.scripts.keys().collect();

    let selector = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .items(&scripts)
        .default(0)
        .interact_on_opt(&dialoguer::console::Term::stdout())
        .expect("Failed to display options");

    let selection = match selector {
        Some(index) => index,
        None => {
            println!("None");
            return;
        }
    };

    let mut child = Command::new("npm")
        .arg("run")
        .arg(scripts[selection])
        .stdout(Stdio::inherit())
        .spawn()
        .expect("err");

    child.wait().expect("err");
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    scripts: HashMap<String, String>,
}
