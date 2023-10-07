mod select;

use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{read_to_string, File};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::{collections::HashMap, process};

use select::Select;

fn main() {
    let args = App::new("rnpm")
        .about("Find the script you're looking for.")
        .arg(
            Arg::with_name("script")
                .takes_value(false)
                .required(false)
                .help("The script you're trying to run. If none provided, you can select the desired on from a list of all the available ones."),
        )
        .arg(
            Arg::with_name("manager")
                .short("m")
                .long("manager")
                .value_name("PACKAGE MANAGER")
                .takes_value(true)
                .possible_values(&["npm", "yarn", "pnpm", "bun"])
                .required(false),
        )
        .arg(
            Arg::with_name("run-exact")
                .short("r")
                .long("run-exact")
                .takes_value(false)
                .required(false)
                .help("If set, immediately run the script that matches exactly."),
        )
        .get_matches();

    let cwd = current_dir().expect("Couldn't access current directory");
    let mut package_json_path = cwd.clone();
    package_json_path.push("package.json");

    let possible_search_arg = args.value_of("script");
    let package_manager =
        find_lockfile_up_tree(cwd).expect("could not find a valid lockfile up the tre");
    let run_exact_flag = args.is_present("run-exact");

    let package_json_contents =
        read_to_string(package_json_path).expect("Could not find package.json file.");

    let package_json: PackageJson =
        serde_json::from_str(&package_json_contents[..]).expect("package.json is not valid JSON.");

    let mut scripts: Vec<&String> = match possible_search_arg {
        Some(arg) => package_json
            .scripts
            .keys()
            .filter(|script| {
                if script == &arg && run_exact_flag {
                    execute_command(&package_manager, arg);
                }

                script.contains(arg)
            })
            .collect(),
        None => package_json.scripts.keys().collect(),
    };

    if scripts.len() == 0 {
        println!("Could not find any scripts.");
        std::process::exit(0);
    } else if scripts.len() == 1 {
        println!("Running script: {}", scripts[0]);

        execute_command(&package_manager, scripts[0]);
    }

    scripts.sort_by(|a, b| a.cmp(b));

    let selected_script = Select::new(&scripts)
        .display()
        .expect("Failed to display options.");

    println!("\r");
    println!("Running script: {}", selected_script);
    println!("\r");

    execute_command(&package_manager, selected_script);
}

fn execute_command(package_manager: &PackageManager, command: &str) -> ! {
    let mut child = Command::new(package_manager)
        .arg("run")
        .arg(command)
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn child process.");

    child.wait().expect("Child Process failed");

    process::exit(0);
}

#[derive(Debug, PartialEq)]
enum PackageManager {
    Npm(String),
    Pnpm(String),
    Yarn(String),
    Bun(String),
}

#[derive(Debug, PartialEq, Eq)]
struct ParseManagerError;

impl FromStr for PackageManager {
    type Err = ParseManagerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "npm" => Ok(Self::Npm(String::from("npm"))),
            "pnpm" => Ok(Self::Pnpm(String::from("pnpm"))),
            "yarn" => Ok(Self::Yarn(String::from("yarn"))),
            "bun" => Ok(Self::Bun(String::from("bun"))),
            _ => Err(ParseManagerError),
        }
    }
}

impl AsRef<OsStr> for PackageManager {
    fn as_ref(&self) -> &OsStr {
        match self {
            PackageManager::Npm(v) => OsStr::new(v),
            PackageManager::Pnpm(v) => OsStr::new(v),
            PackageManager::Yarn(v) => OsStr::new(v),
            PackageManager::Bun(v) => OsStr::new(v),
        }
    }
}

fn find_lockfile_up_tree<P: AsRef<Path>>(path_like: P) -> Option<PackageManager> {
    let managers = [
        ("npm", "package-lock.json"),
        ("yarn", "yarn.lock"),
        ("pnpm", "pnpm-lock.yaml"),
        ("bun", "bun.lockb"),
    ];
    let path = path_like.as_ref().to_path_buf();

    for (manager, lockfile) in managers.iter() {
        let mut lockfile_path = path.clone();
        lockfile_path.push(lockfile);
        let has_lockfile = File::open(lockfile_path).is_ok();

        if has_lockfile {
            return PackageManager::from_str(manager).ok();
        }
    }

    match path.parent() {
        Some(p) => find_lockfile_up_tree(p),
        None => None,
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    scripts: HashMap<String, String>,
}
