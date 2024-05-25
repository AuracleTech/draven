use crate::Draven;
use std::error::Error;

pub enum NodeKind {
    Struct,
    Enum,
    // Trait,
}

pub struct Node {
    pub kind: NodeKind,
    pub fields: Vec<Node>,
}

impl Draven {
    pub fn parse_input(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
