use super::Draven;
use std::{env, error::Error, fs, path::PathBuf, process};

impl Draven {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut input = None;
        let mut output = None;
        let mut watching = false;
        let mut silent = false;
        let mut primitives = false;

        let args: Vec<String> = env::args().skip(1).collect();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-i" => input = iter.next().map(PathBuf::from),
                "-o" => output = iter.next().map(PathBuf::from),
                "-h" => print_help(),
                "-w" => watching = true,
                "-p" => primitives = true,
                "-s" => silent = true,
                _ => Err(format!("Unknown argument: {}", arg))?,
            }
        }

        let input = input.ok_or("No input -i folder provided")?;
        let mut output = output.ok_or("No output -o folder provided")?;

        if !input.exists() {
            Err("Input folder does not exist")?;
        }
        if !output.exists() {
            Err("Output folder does not exist")?;
        }
        output.push("draven_generated");
        fs::create_dir_all(&output)?;

        Ok(Self {
            input,
            output,
            watching,
            silent,
            primitives,

            nodes: Vec::new(),
        })
    }
}

fn print_help() {
    println!(
        r#"Usage: draven -i <input_folder> -o <output_folder>
-i <folder>: location to get rust project from
-o <folder>: location to write markdown files to
-h: Display help message
-w: Watches for file change in input folder
-p: Enable linking primitive types in markdown files
-s: Silent mode"#
    );
    process::exit(0);
}
