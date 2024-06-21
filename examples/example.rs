use draven::parser::{self};
use log::debug;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    error::Error,
    fs::{self},
    io::Write,
    path::PathBuf,
};
use toml::Value;

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let rust_project = PathBuf::from("C:/Users/Silco/Desktop/pulsar");
    let output = PathBuf::from("C:/Users/Silco/Desktop/draven/generated");
    let watching = false;
    let primitives = false;
    let entry = "lib.rs";

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
    let package_name = package_name.trim_matches('"');

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

        if !primitives {
            // IMPLEMENT iterate over module to remove primitives
        }

        let filename = format!("{}.md", package_name);
        let output = output.join(filename);
        debug!("Writing to {:?}", output);

        let mut file = fs::File::create(output)?;
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
