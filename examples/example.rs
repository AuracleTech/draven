use draven::project::parser::Parser;
use log::debug;
use std::{error::Error, path::PathBuf};

pub fn main() -> Result<(), Box<dyn Error>> {
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let mut parser = Parser::new(PathBuf::from("C:/Users/Silco/Desktop/pulsar"))?;
    parser.parse_in_src(&PathBuf::from("lib.rs"))?;
    debug!("{:#?}", parser);

    Ok(())
}
