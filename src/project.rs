pub mod parser;

use std::{error::Error, fs, path::PathBuf};
use toml::Value;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub src: PathBuf,
}

impl Project {
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        if !root.exists() || !root.is_dir() {
            Err("Project root does not exist or is not a folder")?;
        }

        let src = root.join("src");
        if !src.exists() || !src.is_dir() {
            Err("src folder does not exist or is not a folder")?;
        }

        let cargo_toml = root.join("Cargo.toml");
        if !cargo_toml.exists() || !cargo_toml.is_file() {
            Err("Cargo.toml does not exist or is not a file")?;
        }

        let cargo_toml_content = fs::read_to_string(cargo_toml)?;
        let cargo_toml_value: Value = toml::from_str(&cargo_toml_content)?;
        let package = cargo_toml_value
            .get("package")
            .ok_or("Failed to find [package] in Cargo.toml")?;
        let name = package
            .get("name")
            .ok_or("Failed to find name field in [package] from Cargo.toml")?
            .to_string()
            .trim_matches('"')
            .to_string();

        Ok(Self { name, src })
    }
}
