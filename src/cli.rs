use std::{env, error::Error, path::PathBuf, process};

pub struct DravenCLI {
    pub input: PathBuf,
    pub output: PathBuf,
    pub watching: bool,
    pub silent: bool,
    pub primitives: bool,
}

impl DravenCLI {
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
        output.push("draven_generated");

        Ok(Self {
            input,
            output,
            watching,
            silent,
            primitives,
        })
    }
}

fn print_help() {
    println!("Usage: draven -i <input_folder> -o <output_folder>");
    println!("-i <folder>: location to get rust project from");
    println!("-o <folder>: location to write markdown files to");
    println!("-h: Display help message");
    println!("-w: Watches for file change in input folder");
    println!("-p: Enable linking primitive types in markdown files");
    println!("-s: Silent mode");
    process::exit(0);
}
