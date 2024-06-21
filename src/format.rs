use crate::parser::{Constant, Enum, File, Function, Macro, Module, Struct, Trait, TypeAlias};
use std::fmt;

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.folder.join(&self.file_stem).with_extension("rs");
        writeln!(f, "File path: {:?}", path)?;
        for (id, module) in &self.modules {
            writeln!(f, "{} : {}", id, module.to_string())?;
        }
        Ok(())
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module name: {}", self.name)?;
        for (id, function) in &self.functions {
            writeln!(f, "{} : {}", id, function.to_string())?;
        }
        for (id, struct_) in &self.structs {
            writeln!(f, "{} : {}", id, struct_.to_string())?;
        }
        for (id, enum_) in &self.enums {
            writeln!(f, "{} : {}", id, enum_.to_string())?;
        }
        for (id, trait_) in &self.traits {
            writeln!(f, "{} : {}", id, trait_.to_string())?;
        }
        for (id, constant) in &self.constants {
            writeln!(f, "{} : {}", id, constant.to_string())?;
        }
        for (id, macro_) in &self.macros {
            writeln!(f, "{} : {}", id, macro_.to_string())?;
        }
        for (id, import) in &self.imports {
            writeln!(f, "{} : {}", id, import)?;
        }
        for (id, submodule) in &self.submodules {
            writeln!(f, "{} : {}", id, submodule.to_string())?;
        }
        Ok(())
    }
}

impl fmt::Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Trait name: {}", self.name)?;
        for (id, function) in &self.functions {
            writeln!(f, "{} : {}", id, function.to_string())?;
        }
        Ok(())
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Constant name: {}", self.name)?;
        write!(f, "Constant type: {}", self.type_name)?;
        writeln!(f, "Constant value: {}", self.value)?;
        Ok(())
    }
}

impl fmt::Display for Macro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Macro name: {}", self.name)?;
        Ok(())
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Function name: {}", self.name)?;
        for arg in &self.args {
            write!(f, "arg @ {}", arg.to_string())?;
        }
        Ok(())
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Enum name: {}", self.name)?;
        for variant in &self.variants {
            writeln!(f, "[[{}]]", variant)?;
        }
        Ok(())
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for field in &self.fields {
            write!(f, "{}", field.to_string())?;
        }
        for (id, method) in &self.methods {
            writeln!(f, "{} : {}", id, method.to_string())?;
        }
        Ok(())
    }
}

impl fmt::Display for TypeAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: [[{}]]", self.alias, self.type_name)
    }
}
