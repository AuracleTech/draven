use super::Project;
use log::debug;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    path::PathBuf,
};
use syn::{Item, UseTree};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct RustItem {
    pub global_path: String,
}

#[derive(Debug)]
pub struct Parser {
    pub project: Project,
    pub project_items: HashSet<RustItem>,

    current_module_path: PathBuf,
    current_imports: HashMap<String, String>,
}

impl Parser {
    pub fn new(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            project: Project::new(root)?,
            project_items: HashSet::new(),
            current_module_path: PathBuf::new(),
            current_imports: HashMap::new(),
        })
    }

    pub fn parse_in_src(&mut self, file: &PathBuf) -> Result<(), Box<dyn Error>> {
        let path = self.project.src.join(self.current_module_path.join(file));
        if !path.exists() || !path.is_file() {
            Err("File does not exist or is not a file")?;
        }

        debug!("Parsing {:?}", path);

        let content = fs::read_to_string(path)?;
        let syntax = syn::parse_file(&content)?;

        for item in syntax.items {
            match item {
                Item::Mod(item_mod) => self.parse_mod(item_mod)?,
                Item::Use(item_use) => self.parse_use(item_use)?,
                Item::Struct(item_struct) => self.parse_struct(item_struct)?,
                Item::Fn(item_fn) => self.parse_fn(item_fn)?,
                Item::Type(item_type) => self.parse_type(item_type)?,
                Item::Enum(item_enum) => self.parse_enum(item_enum)?,
                _ => {}
            }
        }

        Ok(())
    }

    fn parse_mod(&mut self, item_mod: syn::ItemMod) -> Result<(), Box<dyn Error>> {
        let mod_name = item_mod.ident.to_string();

        let no_mod_path = self
            .project
            .src
            .join(self.current_module_path.join(format!("{}.rs", mod_name)));

        self.current_module_path.push(&mod_name);

        let mod_path = self
            .project
            .src
            .join(self.current_module_path.join("mod.rs"));

        let path = if no_mod_path.exists() && no_mod_path.is_file() {
            Some(no_mod_path)
        } else if mod_path.exists() && mod_path.is_file() {
            Some(mod_path)
        } else {
            None
        };

        if let Some(path) = path {
            self.parse_in_src(&path)?;
        } else {
            return Err(format!("Module {} not found", mod_name).into());
        }

        debug!("Current imports: {:?}", self.current_imports);
        self.current_imports.clear();

        self.current_module_path.pop();

        Ok(())
    }

    fn parse_use(&mut self, item_use: syn::ItemUse) -> Result<(), Box<dyn Error>> {
        fn process_use_tree(
            use_tree: &UseTree,
            prefix: String,
            current_imports: &mut HashMap<String, String>,
        ) {
            match use_tree {
                UseTree::Path(use_path) => {
                    let new_prefix = if prefix.is_empty() {
                        use_path.ident.to_string()
                    } else {
                        format!("{}::{}", prefix, use_path.ident)
                    };
                    process_use_tree(&*use_path.tree, new_prefix, current_imports);
                }
                UseTree::Name(use_name) => {
                    let import_name = if prefix.is_empty() {
                        use_name.ident.to_string()
                    } else {
                        format!("{}::{}", prefix, use_name.ident)
                    };
                    current_imports.insert(import_name.clone(), import_name);
                }
                UseTree::Rename(use_rename) => {
                    let import_name = if prefix.is_empty() {
                        use_rename.ident.to_string()
                    } else {
                        format!("{}::{}", prefix, use_rename.ident)
                    };
                    current_imports.insert(import_name, use_rename.rename.to_string());
                }
                UseTree::Glob(_) => {}
                UseTree::Group(use_group) => {
                    for tree in &use_group.items {
                        process_use_tree(tree, prefix.clone(), current_imports);
                    }
                }
            }
        }

        process_use_tree(&item_use.tree, String::new(), &mut self.current_imports);

        Ok(())
    }

    fn parse_struct(&mut self, item_struct: syn::ItemStruct) -> Result<(), Box<dyn Error>> {
        let struct_name = item_struct.ident.to_string();
        let global_path = self.resolve_path(struct_name.clone());
        let struct_type = RustItem { global_path };
        self.project_items.insert(struct_type);
        Ok(())
    }

    fn resolve_path(&self, local_path: String) -> String {
        let mut global_path = self.current_module_path.clone();
        global_path.push(local_path);
        let mut resolved_path = global_path
            .to_str()
            .unwrap()
            .replace("\\", "::")
            .replace("/", "::");

        if resolved_path.starts_with("crate::") {
            resolved_path = resolved_path.replace("crate::", "");
        }

        while resolved_path.contains("super::") {
            if let Some(super_pos) = resolved_path.find("super::") {
                let pre_super = &resolved_path[0..super_pos];
                let post_super = &resolved_path[(super_pos + 7)..];
                let pre_super_split: Vec<&str> = pre_super.rsplitn(2, "::").collect();
                resolved_path = if pre_super_split.len() > 1 {
                    format!("{}{}", pre_super_split[1], post_super)
                } else {
                    post_super.to_string()
                };
            }
        }

        resolved_path
    }

    fn parse_fn(&mut self, item_fn: syn::ItemFn) -> Result<(), Box<dyn Error>> {
        let fn_name = item_fn.sig.ident.to_string();
        let global_path = self.resolve_path(fn_name.clone());
        let fn_type = RustItem { global_path };
        self.project_items.insert(fn_type);
        Ok(())
    }

    fn parse_type(&mut self, item_type: syn::ItemType) -> Result<(), Box<dyn Error>> {
        let type_name = item_type.ident.to_string();
        let global_path = self.resolve_path(type_name.clone());
        let type_type = RustItem { global_path };
        self.project_items.insert(type_type);
        Ok(())
    }

    fn parse_enum(&mut self, item_enum: syn::ItemEnum) -> Result<(), Box<dyn Error>> {
        let enum_name = item_enum.ident.to_string();
        let global_path = self.resolve_path(enum_name.clone());
        let enum_type = RustItem { global_path };
        self.project_items.insert(enum_type);
        Ok(())
    }
}
