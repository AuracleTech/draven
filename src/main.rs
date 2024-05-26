mod cli;
mod parser;

use notify::RecursiveMode;
use parser::{Function, Struct};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

pub struct Draven {
    pub input: PathBuf,
    pub output: PathBuf,
    pub watching: bool,
    pub silent: bool,
    pub primitives: bool,
    pub recursive: RecursiveMode,

    structs: HashMap<String, Struct>,
    functions: HashMap<String, Function>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut draven = Draven::new()?;

    draven.parse_path(draven.input.clone().as_path())?;

    if draven.watching {
        loop {
            draven.watch()?;
        }
    }

    Ok(())
}
