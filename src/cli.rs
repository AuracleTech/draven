use super::Draven;
use notify::RecursiveMode;
use std::{env, error::Error, fs, path::PathBuf, process};

impl Draven {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut input = None;
        let mut vault = None;
        let mut watching = false;
        let mut silent = false;
        let mut primitives = false;
        let mut recursive = RecursiveMode::NonRecursive;

        let args: Vec<String> = env::args().skip(1).collect();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-i" => input = iter.next().map(PathBuf::from),
                "-o" => vault = iter.next().map(PathBuf::from),
                "-h" => help(),
                "-w" => watching = true,
                "-p" => primitives = true,
                "-s" => silent = true,
                "-r" => recursive = RecursiveMode::Recursive,
                _ => Err(format!("Unknown argument: {}", arg))?,
            }
        }

        let input = input.ok_or("No input -i folder provided")?;
        let vault = vault.ok_or("No output -o folder provided")?;

        if !input.exists() && input.is_dir() {
            Err("Input folder does not exist")?;
        }
        if !vault.exists() && vault.is_dir() {
            Err("Output folder does not exist")?;
        }

        let output = vault.join("draven");

        if output.exists() {
            fs::remove_dir_all(&output)?;
        }

        fs::create_dir_all(&output)?;

        Ok(Self {
            input,
            output,
            watching,
            silent,
            primitives,
            recursive,

            structs: Default::default(),
            functions: Default::default(),
        })
    }
}

fn help() {
    log::info!(
        r#"Usage: draven -i <rust_src_path> -o <obsidian_vault_path>
-h: Display help message
-i <src_path>: location to get rust project src from
-o <vault_path>: location to write the obsidian files to
-w: Watches for file change in real time to update obsidian files
-r: Enable recursive folder processing in input folder
-p: Enable primitive types linking in markdown files
-s: Silent mode"#
    );
    process::exit(0);
}
