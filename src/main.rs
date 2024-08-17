use clap::{crate_name, crate_version, Parser};
use dont_disappear::any_key_to_continue;
use mslnk::ShellLink;
use std::{
    env::var_os,
    path::{Path, PathBuf},
    process::{exit, Command},
};

#[derive(Parser)]
#[command()]
pub struct Cli {
    /// Print version
    #[arg(long)]
    pub version: bool,
    pub paths: Vec<String>,
}

fn main() {
    let parsed = Cli::parse();
    if parsed.version {
        println!("{} {}", crate_name!(), crate_version!());
        return;
    }

    let Some(appdata) = var_os("APPDATA") else {
        eprintln!("Cannot determine your appdata directory");
        exit(1);
    };
    let mut start_menu = PathBuf::from(appdata);
    start_menu.push(r"Microsoft\Windows\Start Menu\Programs");
    if !start_menu.exists() {
        eprintln!("Cannot locate your start menu directory");
        exit(1);
    }

    let mut errs = Vec::new();
    for input in &parsed.paths {
        let path = Path::new(&input);
        if let Some(err) = is_invalid_file(&path) {
            errs.push(err);
            continue;
        };
        let mut name = path.file_stem().unwrap().to_owned();
        name.push(".lnk");
        let target: PathBuf = [start_menu.as_os_str(), &name].iter().collect();
        if let Err(err) = ShellLink::new(path).unwrap().create_lnk(&target) {
            errs.push(err.to_string());
        } else if parsed.paths.len() == 1 {
            Command::new("explorer.exe")
                .args(["/select,", &target.to_string_lossy()])
                .spawn()
                .unwrap();
        }
    }

    if parsed.paths.len() != 1 || parsed.paths.len() != errs.len() {
        Command::new("explorer.exe")
            .args([start_menu])
            .spawn()
            .unwrap();
    }
    if errs.len() > 0 {
        for err in errs {
            eprintln!("{err}");
        }
        any_key_to_continue::default();
        exit(1);
    }
}

pub fn is_invalid_file(path: &Path) -> Option<String> {
    if !path.exists() {
        Some(format!(r#""{}" does not exist!"#, path.to_string_lossy()))
    } else if path.extension().is_some_and(|val| val == "exe") {
        None
    } else {
        Some(format!(
            r#""{}" must be an ".exe"!"#,
            path.to_string_lossy()
        ))
    }
}
