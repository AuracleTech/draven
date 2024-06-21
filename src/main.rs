use draven::parser::{self};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process,
};
use toml::Value;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut rust_project = None;
    let mut output = None;
    let mut watching = false;
    let mut entry = "main.rs";

    let args: Vec<String> = env::args().skip(1).collect();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" => rust_project = iter.next().map(PathBuf::from),
            "-o" => output = iter.next().map(PathBuf::from),
            "-w" => watching = true,
            "-e" => entry = iter.next().ok_or("No entry after -e")?,
            "-h" => {
                log::info!(
                    r#"Usage: draven -i <rust_src_path> -o <obsidian_vault_path>
-i <src_path>: location to get rust project src from
-o <vault_path>: location to write the obsidian files to
-w: Watches for file change in real time to update obsidian files
-p: Enable primitive types linking in markdown files
-e <entry_file>: Entry file to start parsing from
-h: Display help message"#
                );
                process::exit(0);
            }
            _ => Err(format!("Unknown argument: {}", arg))?,
        }
    }

    let rust_project = rust_project.ok_or("No input -i folder provided")?;
    let output = output.ok_or("No output -o folder provided")?;

    if !rust_project.exists() {
        Err("Input folder does not exist")?;
    }
    if !rust_project.is_dir() {
        Err("Input is not a folder")?;
    }
    if !output.exists() {
        Err("Output folder does not exist")?;
    }
    if !output.is_dir() {
        Err("Output is not a folder")?;
    }

    let project_src = rust_project.join("src");
    if !project_src.exists() {
        Err("Folder \"src\" for source does not exist")?;
    }
    let project_src_entry = project_src.join(entry);
    if !project_src_entry.exists() {
        Err(format!("Entry file {:?} does not exist", project_src_entry))?;
    }

    let cargo_toml = rust_project.join("Cargo.toml");
    if !cargo_toml.exists() {
        Err("Cargo.toml not found in input folder, is this a rust project?")?;
    }

    let cargo_toml_content = fs::read_to_string(cargo_toml)?;
    let cargo_toml_value: Value = toml::from_str(&cargo_toml_content)?;
    let package = cargo_toml_value
        .get("package")
        .ok_or("Failed to find [package] in Cargo.toml")?;
    let package_name = package
        .get("name")
        .ok_or("Failed to find name field in [package] from Cargo.toml")?;
    let package_name = package_name.to_string();

    loop {
        log::info!("Parsing project: {}", package_name);

        let project_src_entry_stem = project_src_entry
            .file_stem()
            .expect("Failed to get path stem")
            .to_str()
            .expect("Failed to convert path stem to string")
            .to_string();
        let project_src_entry_folder = project_src_entry
            .parent()
            .expect("Failed to get parent folder")
            .to_path_buf();
        let mut file_new = parser::File::new(project_src_entry_stem, project_src_entry_folder);
        file_new.parse()?;

        let filename = format!("{}.md", package_name);
        let output = output.join(filename);
        let mut file = File::create(output)?;
        writeln!(file, "{}", file_new)?;

        if watching {
            log::info!("Watching for file changes in {:?}...", rust_project);

            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
            watcher.watch(&project_src, RecursiveMode::Recursive)?;

            for result in rx {
                match result {
                    Ok(event) => {
                        if let Some(path) = event.paths.first() {
                            if let Some(extension) = path.extension() {
                                if extension == "rs" {
                                    continue;
                                }
                            }
                        }
                    }
                    Err(error) => Err(error)?,
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}
