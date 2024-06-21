use log::debug;
use quote::ToTokens;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};
use syn::{Fields, Item, UseTree};

#[derive(Debug)]
pub struct TypeAlias {
    pub alias: String,
    pub type_name: String,
}

#[derive(Debug)]
pub struct Struct {
    pub type_name: String,
    pub fields: Vec<TypeAlias>,
    pub methods: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<TypeAlias>,
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub type_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Trait {
    pub name: String,
    pub functions: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct Macro {
    pub name: String,
    pub body: String,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub functions: HashMap<String, Function>,
    pub structs: HashMap<String, Struct>,
    pub enums: HashMap<String, Enum>,
    pub traits: HashMap<String, Trait>,
    pub constants: HashMap<String, Constant>,
    pub macros: HashMap<String, Macro>,
    pub imports: HashMap<String, String>,
    pub submodules: HashMap<String, Module>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            constants: HashMap::new(),
            macros: HashMap::new(),
            imports: HashMap::new(),
            submodules: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct File {
    pub folder: PathBuf,
    pub file_stem: String,
    pub modules: HashMap<String, Module>,
}

impl File {
    pub fn new(file_stem: String, folder: PathBuf) -> Self {
        Self {
            folder,
            file_stem,
            modules: HashMap::new(),
        }
    }

    pub fn parse(&mut self) -> Result<(), Box<dyn Error>> {
        let fullpath = self.folder.join(&self.file_stem).with_extension("rs");
        debug!("File::parse({:?})", fullpath);
        let content = fs::read_to_string(fullpath)?;
        let syntax = syn::parse_file(&content)?;

        let mut root_module = Module::new(self.file_stem.clone());

        for item in syntax.items {
            File::parse_item(&mut root_module, item, &self.folder)?;
        }

        self.modules.insert(root_module.name.clone(), root_module);
        Ok(())
    }

    fn parse_item(module: &mut Module, item: Item, base_path: &Path) -> Result<(), Box<dyn Error>> {
        match item {
            Item::Use(item_use) => File::parse_use_tree(&item_use.tree, String::new(), module)?,
            Item::Struct(item_struct) => module.parse_struct(item_struct)?,
            Item::Fn(item_fn) => module.parse_function(item_fn)?,
            Item::Impl(item_impl) => module.parse_impl(item_impl)?,
            Item::Enum(item_enum) => module.parse_enum(item_enum)?,
            Item::Const(item_const) => module.parse_constant(item_const)?,
            Item::Type(item_type) => module.parse_type_alias(item_type)?,
            Item::Mod(item_mod) => {
                module.parse_submodule(item_mod, base_path, &module.name.clone())?
            }
            Item::Macro(item_macro) => module.parse_macro(item_macro)?,
            Item::Trait(item_trait) => module.parse_trait(item_trait)?,
            _ => {
                log::warn!("Unsupported syn item received: {:?}", item);
            }
        }
        Ok(())
    }

    fn parse_use_tree(
        tree: &UseTree,
        prefix: String,
        module: &mut Module,
    ) -> Result<(), Box<dyn Error>> {
        match tree {
            UseTree::Path(use_path) => {
                let new_prefix = if prefix.is_empty() {
                    use_path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, use_path.ident)
                };
                File::parse_use_tree(&use_path.tree, new_prefix, module)?;
            }
            UseTree::Name(use_name) => {
                let full_path = if prefix.is_empty() {
                    use_name.ident.to_string()
                } else {
                    format!("{}::{}", prefix, use_name.ident)
                };
                module.imports.insert(use_name.ident.to_string(), full_path);
            }
            UseTree::Group(use_group) => {
                for item in &use_group.items {
                    File::parse_use_tree(item, prefix.clone(), module)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Module {
    fn parse_struct(&mut self, item_struct: syn::ItemStruct) -> Result<(), Box<dyn Error>> {
        let fields = self.parse_fields(&item_struct.fields)?;

        self.structs.insert(
            item_struct.ident.to_string(),
            Struct {
                type_name: item_struct.ident.to_string(),
                fields,
                methods: HashMap::new(),
            },
        );
        Ok(())
    }

    fn parse_fields(&self, fields: &Fields) -> Result<Vec<TypeAlias>, Box<dyn Error>> {
        let mut type_alias_list = Vec::new();
        for field in fields {
            let alias = match &field.ident {
                Some(ident) => ident.to_string(),
                None => "".to_string(),
            };

            let type_name = match &field.ty {
                syn::Type::Path(ref type_path) => type_path
                    .path
                    .segments
                    .last()
                    .expect("Failed to get last segment")
                    .ident
                    .to_string(),
                _ => "".to_string(),
            };

            type_alias_list.push(TypeAlias { alias, type_name });
        }
        Ok(type_alias_list)
    }

    fn parse_function(&mut self, item_fn: syn::ItemFn) -> Result<(), Box<dyn Error>> {
        let name = item_fn.sig.ident.to_string();
        let args = item_fn
            .sig
            .inputs
            .iter()
            .map(|fn_arg| {
                let alias = match fn_arg {
                    syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                };

                let type_name = match fn_arg {
                    syn::FnArg::Typed(pat_type) => pat_type.ty.to_token_stream().to_string(),
                    _ => "".to_string(),
                };

                TypeAlias { alias, type_name }
            })
            .collect();

        let node = Function {
            name: name.clone(),
            args,
        };
        self.functions.insert(name, node);
        Ok(())
    }

    fn parse_impl(&mut self, item_impl: syn::ItemImpl) -> Result<(), Box<dyn Error>> {
        let parent = match *item_impl.self_ty {
            syn::Type::Path(ref type_path) => type_path
                .path
                .segments
                .last()
                .expect("Failed to get last segment")
                .ident
                .to_string(),
            _ => return Err("Unsupported type in impl block".into()),
        };

        if !self.structs.contains_key(&parent) {
            log::warn!("Parent struct '{}' not found for impl block", parent);
            return Ok(());
        }

        for item in item_impl.items {
            self.parse_method(&item, &parent)?;
        }

        Ok(())
    }

    fn parse_method(&mut self, item: &syn::ImplItem, parent: &str) -> Result<(), Box<dyn Error>> {
        if let syn::ImplItem::Fn(impl_item_method) = item {
            let name = impl_item_method.sig.ident.to_string();
            let args = impl_item_method
                .sig
                .inputs
                .iter()
                .map(|fn_arg| {
                    let alias = match fn_arg {
                        syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                            syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                            _ => "".to_string(),
                        },
                        _ => "".to_string(),
                    };

                    let type_name = match fn_arg {
                        syn::FnArg::Typed(pat_type) => pat_type.ty.to_token_stream().to_string(),
                        _ => "".to_string(),
                    };

                    TypeAlias { alias, type_name }
                })
                .collect();

            if let Some(structure) = self.structs.get_mut(parent) {
                let node = Function {
                    name: name.clone(),
                    args,
                };
                structure.methods.insert(name, node);
            } else {
                log::warn!(
                    "Failed to find parent struct '{}' for method '{}'",
                    parent,
                    name
                );
            }
        }
        Ok(())
    }

    fn parse_enum(&mut self, item_enum: syn::ItemEnum) -> Result<(), Box<dyn Error>> {
        let type_name = item_enum.ident.to_string();
        let variants = item_enum
            .variants
            .iter()
            .map(|variant| variant.ident.to_string())
            .collect();

        let node = Enum {
            name: type_name.clone(),
            variants,
        };
        self.enums.insert(type_name, node);
        Ok(())
    }

    fn parse_constant(&mut self, item_const: syn::ItemConst) -> Result<(), Box<dyn Error>> {
        let name = item_const.ident.to_string();
        let value = item_const.expr.to_token_stream().to_string();

        let node = Constant {
            name: name.clone(),
            type_name: item_const.ty.to_token_stream().to_string(),
            value,
        };
        self.constants.insert(name, node);
        Ok(())
    }

    fn parse_type_alias(&mut self, item_type: syn::ItemType) -> Result<(), Box<dyn Error>> {
        let alias = item_type.ident.to_string();
        let type_name = item_type.ty.to_token_stream().to_string();

        let node = TypeAlias { alias, type_name };

        if let Some(parent) = self.structs.get_mut(&node.type_name) {
            parent.fields.push(node);
        } else {
            log::warn!(
                "Failed to find parent struct '{}' for type alias '{}'",
                node.type_name,
                node.alias
            );
        }

        Ok(())
    }

    fn parse_submodule(
        &mut self,
        item_mod: syn::ItemMod,
        base_path: &Path,
        parent: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mod_name = item_mod.ident.to_string();

        if let Some((_, content)) = item_mod.content {
            debug!("Parsing inline submodule: {:?}", mod_name);
            let mut submodule = Module {
                name: mod_name.clone(),
                functions: HashMap::new(),
                structs: HashMap::new(),
                enums: HashMap::new(),
                traits: HashMap::new(),
                constants: HashMap::new(),
                macros: HashMap::new(),
                submodules: HashMap::new(),
                imports: HashMap::new(),
            };

            for item in content {
                File::parse_item(&mut submodule, item, &base_path.join(&mod_name))?;
            }

            self.submodules.insert(mod_name, submodule);
        } else {
            let paths = [
                base_path.join(&mod_name).with_extension("rs"),
                base_path.join(&mod_name).join("mod.rs"),
                base_path.join(parent).join(&mod_name).with_extension("rs"),
                base_path.join(parent).join(&mod_name).join("mod.rs"),
            ];

            let submodule_file = paths
                .iter()
                .find_map(|path| {
                    if path.exists() {
                        debug!("Parsing file submodule: {:?}", path);
                        let canonical_path = path.canonicalize().ok()?;

                        let canonical_path_stem = canonical_path
                            .file_stem()
                            .expect("Failed to get path stem")
                            .to_str()
                            .expect("Failed to convert path stem to string")
                            .to_string();
                        let canonical_path_folder = canonical_path
                            .parent()
                            .expect("Failed to get parent folder")
                            .to_path_buf();

                        let mut file = File::new(canonical_path_stem, canonical_path_folder);
                        file.parse().ok()?;
                        Some(file)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| format!("Submodule file not found for module '{}'", mod_name))?;

            if let Some((_, module)) = submodule_file.modules.into_iter().next() {
                self.submodules.insert(mod_name, module);
            } else {
                return Err(format!("Failed to parse submodule '{}'", mod_name).into());
            }
        }

        Ok(())
    }

    fn parse_macro(&mut self, item_macro: syn::ItemMacro) -> Result<(), Box<dyn Error>> {
        let name = item_macro
            .mac
            .path
            .segments
            .last()
            .expect("Failed to get last segment")
            .ident
            .to_string();
        let body = item_macro.mac.tokens.to_string();

        let node = Macro {
            name: name.clone(),
            body,
        };
        self.macros.insert(name, node);
        Ok(())
    }

    fn parse_trait(&mut self, item_trait: syn::ItemTrait) -> Result<(), Box<dyn Error>> {
        let name = item_trait.ident.to_string();
        let functions = item_trait
            .items
            .iter()
            .filter_map(|item| {
                if let syn::TraitItem::Fn(method) = item {
                    Some(Function {
                        name: method.sig.ident.to_string(),
                        args: method
                            .sig
                            .inputs
                            .iter()
                            .map(|fn_arg| {
                                let alias = match fn_arg {
                                    syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                                        syn::Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                                        _ => "".to_string(),
                                    },
                                    _ => "".to_string(),
                                };

                                let type_name = match fn_arg {
                                    syn::FnArg::Typed(pat_type) => {
                                        pat_type.ty.to_token_stream().to_string()
                                    }
                                    _ => "".to_string(),
                                };

                                TypeAlias { alias, type_name }
                            })
                            .collect(),
                    })
                } else {
                    None
                }
            })
            .map(|function| (function.name.clone(), function))
            .collect();

        let node = Trait {
            name: name.clone(),
            functions,
        };
        self.traits.insert(name, node);
        Ok(())
    }
}
