use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::{env, process};
use std::{fs, path::Path};
use syn::{Item, ItemUse, UseTree};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (mut src_dir, mut output_dir) = (String::new(), String::new());
    let (mut watching, mut silent, mut primitives) = (false, false, false);

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" | "-input" | "--input" => src_dir = verify_folder(iter.next(), "-i")?,
            "-o" | "-output" | "--output" => output_dir = verify_folder(iter.next(), "-o")?,
            "-h" | "-help" | "--help" => print_help(),
            "-w" | "-watch" | "--watch" => watching = true,
            "-p" | "-primitives" | "--primitives" => primitives = true,
            "-s" | "-silent" | "--silent" => silent = true,
            _ => Err(format!("Unknown argument: {}", arg))?,
        }
    }

    let src_dir = PathBuf::from(src_dir);
    let mut output_dir = PathBuf::from(output_dir);
    output_dir.push("draven_generated");

    work(&src_dir, &output_dir, silent, primitives)?;

    if watching {
        loop {
            watch(&src_dir, &output_dir, silent, primitives)?;
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: draven -i <input_folder> -o <output_folder>");
    println!("-i <folder>: location to get rust project from");
    println!("-o <folder>: location to write markdown files to");
    println!("-h: Display help message");
    println!("-w: Watches for file change in input folder");
    println!("-p: Enable linking primitive types in markdown files");
    println!("-s: Silent mode");
    process::exit(0);
}

fn work<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &PathBuf,
    silent: bool,
    primitives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;
    traverse_directory(&src_dir, output_dir, primitives)?;
    if !silent {
        println!("Markdown files generated");
    }
    Ok(())
}

fn verify_folder(
    folder: Option<&String>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(folder) = folder {
        let path = Path::new(folder);
        if path.exists() && path.is_dir() {
            return Ok(folder.clone());
        }
        Err(format!("{:?} is not a valid folder", path).into())
    } else {
        Err(format!("Expected argument after {}", flag).into())
    }
}

fn watch<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &PathBuf,
    silent: bool,
    primitives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(src_dir.as_ref(), RecursiveMode::Recursive)?;

    if !silent {
        println!("Now watching for file changes in {:?}...", src_dir.as_ref());
    }

    for res in rx {
        match res {
            Ok(event) => {
                if let Some(path) = event.paths.first() {
                    if let Some(extension) = path.extension() {
                        if extension == "rs" {
                            return work(&src_dir, output_dir, silent, primitives);
                        }
                    }
                }
            }
            Err(error) => Err(error)?,
        }
    }

    Ok(())
}

fn traverse_directory<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &PathBuf,
    primitives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            traverse_directory(&path, output_dir, primitives)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                parse_and_convert_to_markdown(&path, output_dir, primitives)?;
            }
        }
    }
    Ok(())
}
fn resolve_full_type_path(ty: &syn::Type, import_map: &HashMap<String, String>) -> String {
    match ty {
        syn::Type::Path(type_path) => {
            let segments = &type_path.path.segments;
            let mut full_path = String::new();
            if let Some(first_segment) = segments.first() {
                if let Some(mapped_path) = import_map.get(&first_segment.ident.to_string()) {
                    full_path.push_str(mapped_path);
                    for segment in segments.iter().skip(1) {
                        full_path.push_str("::");
                        full_path.push_str(&segment.ident.to_string());
                        if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                            full_path.push('<');
                            for (i, arg) in args.args.iter().enumerate() {
                                if let syn::GenericArgument::Type(ref generic_ty) = arg {
                                    if i > 0 {
                                        full_path.push_str(", ");
                                    }
                                    full_path
                                        .push_str(&resolve_full_type_path(generic_ty, import_map));
                                }
                            }
                            full_path.push('>');
                        }
                    }
                    return full_path;
                }
            }
            segments
                .iter()
                .map(|segment| {
                    let mut segment_str = segment.ident.to_string();
                    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                        segment_str.push_str("<[[");
                        for (i, arg) in args.args.iter().enumerate() {
                            if let syn::GenericArgument::Type(ref generic_ty) = arg {
                                if i > 0 {
                                    segment_str.push_str(", ");
                                }
                                segment_str
                                    .push_str(&resolve_full_type_path(generic_ty, import_map));
                            }
                        }
                        segment_str.push_str("]]>");
                    }
                    segment_str
                })
                .collect::<Vec<_>>()
                .join("::")
        }
        syn::Type::Array(type_array) => {
            resolve_full_type_path(&type_array.elem, import_map).to_string()
        }
        syn::Type::Reference(type_reference) => {
            let mut ref_str = String::new();
            if let Some(lifetime) = &type_reference.lifetime {
                ref_str.push_str(&lifetime.to_string());
                ref_str.push(' ');
            }
            ref_str.push('&');
            if type_reference.mutability.is_some() {
                ref_str.push_str("mut ");
            }
            ref_str.push_str(&resolve_full_type_path(&type_reference.elem, import_map));
            ref_str
        }
        syn::Type::Slice(type_slice) => {
            resolve_full_type_path(&type_slice.elem, import_map).to_string()
        }
        syn::Type::Tuple(type_tuple) => type_tuple
                .elems
                .iter()
                .map(|elem| resolve_full_type_path(elem, import_map))
                .collect::<Vec<_>>()
                .join(", ").to_string(),
        _ => "unknown".to_string(),
    }
}

fn parse_and_convert_to_markdown<P: AsRef<Path>>(
    path: P,
    output_dir: &Path,
    primitives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&path)?;
    let syntax_res = syn::parse_file(&content);
    if syntax_res.is_err() {
        return Ok(());
    }
    let syntax = syntax_res?;

    let mut import_map = HashMap::new();
    for item in &syntax.items {
        if let Item::Use(ItemUse { tree, .. }) = item {
            parse_use_tree(tree, &mut import_map, String::new());
        }
    }

    for item in syntax.items {
        if let Item::Struct(s) = item {
            let struct_name = s.ident.to_string();
            let mut markdown = format!("# {}\n\n", struct_name);
            markdown.push_str("## name: Type\n\n");
            for field in s.fields {
                let field_name = field
                    .ident
                    .as_ref()
                    .map(|ident| ident.to_string())
                    .unwrap_or_else(|| "unnamed_field".to_string());
                let type_name = resolve_full_type_path(&field.ty, &import_map);
                let formatted_type_name = if type_name.contains('[') || type_name.contains(']') {
                    type_name
                } else if primitives {
                    format!("[[{}]]", type_name)
                } else if is_primitive_type(&type_name) {
                    type_name
                } else {
                    format!("[[{}]]", type_name)
                };
                markdown.push_str(&format!("{} : {}\n", field_name, formatted_type_name));
            }

            let output_file = output_dir.join(format!("{}.md", struct_name));
            let mut file = fs::File::create(output_file)?;
            file.write_all(markdown.as_bytes())?;
        }
    }
    Ok(())
}

fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "isize"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
    )
}

fn parse_use_tree(tree: &UseTree, import_map: &mut HashMap<String, String>, prefix: String) {
    match tree {
        UseTree::Path(path) => {
            let new_prefix = if prefix.is_empty() {
                path.ident.to_string()
            } else {
                format!("{}::{}", prefix, path.ident)
            };
            parse_use_tree(&path.tree, import_map, new_prefix);
        }
        UseTree::Name(name) => {
            let full_path = if prefix.is_empty() {
                name.ident.to_string()
            } else {
                format!("{}::{}", prefix, name.ident)
            };
            import_map.insert(name.ident.to_string(), full_path);
        }
        UseTree::Rename(rename) => {
            let full_path = if prefix.is_empty() {
                rename.ident.to_string()
            } else {
                format!("{}::{}", prefix, rename.ident)
            };
            import_map.insert(rename.rename.to_string(), full_path);
        }
        UseTree::Glob(_glob) => {}
        UseTree::Group(group) => {
            for tree in &group.items {
                parse_use_tree(tree, import_map, prefix.clone());
            }
        }
    }
}
