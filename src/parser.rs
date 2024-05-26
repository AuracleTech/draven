use crate::Draven;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::Path,
};
use syn::{Fields, Item, UseTree};

#[derive(Debug)]
pub struct TypeAlias {
    alias: String,
    type_name: String,
}

impl Node for TypeAlias {
    fn filename(&self) -> String {
        self.type_name.clone()
    }

    fn print(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}: [[{}]]\n", self.alias, self.type_name));
        output
    }
}

#[derive(Debug)]
pub struct Struct {
    type_name: String,
    fields: Vec<TypeAlias>,
    methods: Vec<Function>,
}

impl Node for Struct {
    fn filename(&self) -> String {
        self.type_name.clone()
    }

    fn print(&self) -> String {
        let mut output = String::from("#Struct\n\n");
        for field in &self.fields {
            output.push_str(&field.print());
        }

        output.push_str("\nMethods\n");

        for method in &self.methods {
            output.push_str(format!("[[{}]]\n", method.name).as_str());
        }
        output
    }
}

#[derive(Debug)]
pub struct Enum {
    type_name: String,
    variants: Vec<String>,
}

impl Node for Enum {
    fn filename(&self) -> String {
        self.type_name.clone()
    }

    fn print(&self) -> String {
        let mut output = String::from("#Enum\n\n");
        for variant in &self.variants {
            output.push_str(&format!("[[{}]]\n", variant));
        }
        output
    }
}

#[derive(Debug)]
pub struct Function {
    name: String,
    args: Vec<TypeAlias>,
}

impl Node for Function {
    fn filename(&self) -> String {
        self.name.clone()
    }

    fn print(&self) -> String {
        let mut output = String::from("#Function\n\n");
        for arg in &self.args {
            output.push_str(&arg.print());
        }
        output
    }
}

pub trait Node {
    fn filename(&self) -> String;
    fn print(&self) -> String;
    fn generate_markdown(&self, output: &Path) -> Result<(), Box<dyn Error>> {
        let filename = format!("{}.md", self.filename());
        let output = output.join(filename);
        let mut file = File::create(output)?;
        writeln!(file, "{}", self.print())?;
        Ok(())
    }
}

impl Draven {
    pub fn watch(&mut self) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(self.input.as_path(), self.recursive)?;

        if !self.silent {
            log::info!("Watching for file changes in {:?}...", self.input);
        }

        for result in rx {
            match result {
                Ok(event) => {
                    if let Some(path) = event.paths.first() {
                        if let Some(extension) = path.extension() {
                            if extension == "rs" {
                                return self.parse_path(self.input.clone().as_path());
                            }
                        }
                    }
                }
                Err(error) => Err(error)?,
            }
        }

        Ok(())
    }

    pub fn parse_path(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && self.recursive == RecursiveMode::Recursive {
                    self.parse_path(&path)?;
                } else {
                    self.parse_file(&path)?;
                }
            }
        } else {
            self.parse_file(path)?;
        }

        self.markdowns()?;

        Ok(())
    }

    fn parse_file(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let syntax = syn::parse_file(&content)?;

        let mut imports = HashMap::new();

        for item in syntax.items {
            self.parse_item(item, &mut imports)?;
        }

        Ok(())
    }

    fn parse_item(
        &mut self,
        item: Item,
        imports: &mut HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        match item {
            Item::Use(item_use) => Self::parse_use_tree(&item_use.tree, String::new(), imports),
            Item::Struct(item_struct) => self.parse_struct(item_struct),
            Item::Fn(item_fn) => self.parse_function(item_fn),
            Item::Impl(item_impl) => self.parse_impl(item_impl),
            _ => {
                log::warn!("Unsupported syn item received: {:?}", item);
                Ok(())
            }
        }
    }

    fn parse_struct<'b>(&mut self, item_struct: syn::ItemStruct) -> Result<(), Box<dyn Error>> {
        let type_name = item_struct.ident.to_string();
        let fields = self.parse_fields(&item_struct.fields)?;

        let node = Struct {
            type_name: type_name.clone(),
            fields,
            methods: Vec::new(),
        };
        self.structs.insert(type_name, node);
        Ok(())
    }

    fn parse_fields(&self, fields: &Fields) -> Result<Vec<TypeAlias>, Box<dyn Error>> {
        let mut field_nodes = Vec::new();
        for field in fields {
            let alias = match &field.ident {
                Some(ident) => ident.to_string(),
                None => "".to_string(), // Handle tuple struct fields which have no names
            };

            let type_name = match &field.ty {
                syn::Type::Path(ref type_path) => {
                    type_path.path.segments.last().unwrap().ident.to_string()
                }
                _ => "".to_string(),
            };

            field_nodes.push(TypeAlias { alias, type_name });
        }
        Ok(field_nodes)
    }

    fn parse_use_tree(
        tree: &UseTree,
        prefix: String,
        imports: &mut HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        match tree {
            UseTree::Path(use_path) => {
                let new_prefix = if prefix.is_empty() {
                    use_path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, use_path.ident)
                };
                Self::parse_use_tree(&use_path.tree, new_prefix, imports)?;
            }
            UseTree::Name(use_name) => {
                let full_path = if prefix.is_empty() {
                    use_name.ident.to_string()
                } else {
                    format!("{}::{}", prefix, use_name.ident)
                };
                imports.insert(use_name.ident.to_string(), full_path);
            }
            UseTree::Group(use_group) => {
                for item in &use_group.items {
                    Self::parse_use_tree(item, prefix.clone(), imports)?;
                }
            }
            _ => {}
        }
        Ok(())
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

                TypeAlias {
                    alias,
                    type_name: name.clone(),
                }
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
            syn::Type::Path(ref type_path) => {
                type_path.path.segments.last().unwrap().ident.to_string()
            }
            _ => return Err("Unsupported type in impl block".into()),
        };

        for item in item_impl.items {
            self.parse_method(&item, &parent)?;
        }

        // for item in item_impl.items {
        //     if let syn::ImplItem::Fn(impl_item_fn) = item {
        //         let type_name = impl_item_fn.sig.ident.to_string();
        //         let args = impl_item_fn
        //             .sig
        //             .inputs
        //             .iter()
        //             .map(|fn_arg| {
        //                 let alias = match fn_arg {
        //                     syn::FnArg::Typed(pat_type) => match &*pat_type.ty {
        //                         syn::Type::Path(ref type_path) => {
        //                             type_path.path.segments.last().unwrap().ident.to_string()
        //                         }
        //                         _ => "".to_string(),
        //                     },
        //                     _ => "".to_string(),
        //                 };

        //                 TypeAlias {
        //                     alias,
        //                     type_name: type_name.clone(),
        //                 }
        //             })
        //             .collect();

        //         Node {
        //             title: type_name,
        //             kind: NodeKind::Method {
        //                 parent: parent.clone(),
        //                 args,
        //             },
        //         };
        //     }
        // }

        Ok(())
    }

    fn parse_method(&mut self, item: &syn::ImplItem, parent: &str) -> Result<(), Box<dyn Error>> {
        if let syn::ImplItem::Fn(impl_item_method) = item {
            let type_name = impl_item_method.sig.ident.to_string();
            let args = impl_item_method
                .sig
                .inputs
                .iter()
                .map(|fn_arg| {
                    let alias = match fn_arg {
                        syn::FnArg::Typed(pat_type) => match &*pat_type.ty {
                            syn::Type::Path(ref type_path) => {
                                type_path.path.segments.last().unwrap().ident.to_string()
                            }
                            _ => "".to_string(),
                        },
                        _ => "".to_string(),
                    };

                    TypeAlias {
                        alias,
                        type_name: type_name.clone(),
                    }
                })
                .collect();

            let structure = self.structs.get_mut(parent).unwrap();
            let node = Function {
                name: type_name,
                args,
            };
            structure.methods.push(node);
            Ok(())
        } else {
            Err("Unsupported type in impl block".into())
        }
    }

    pub fn markdowns(&mut self) -> Result<(), Box<dyn Error>> {
        for (_key, value) in &self.structs {
            value.generate_markdown(&self.output)?;

            for method in &value.methods {
                method.generate_markdown(&self.output)?;
            }
        }
        for (_key, value) in &self.functions {
            value.generate_markdown(&self.output)?;
        }

        self.structs.clear();
        self.functions.clear();

        Ok(())
    }
}
