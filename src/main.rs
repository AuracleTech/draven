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
    let (mut watching, mut silent) = (false, false);

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" | "-input" | "--input" => src_dir = verify_folder(iter.next(), "-i")?,
            "-o" | "-output" | "--output" => output_dir = verify_folder(iter.next(), "-o")?,
            "-h" | "-help" | "--help" => print_help(),
            "-w" | "-watch" | "--watch" => watching = true,
            "-s" | "-silent" | "--silent" => silent = true,
            _ => Err(format!("Unknown argument: {}", arg))?,
        }
    }

    let src_dir = PathBuf::from(src_dir);
    let mut output_dir = PathBuf::from(output_dir);
    output_dir.push("draven_generated");

    work(&src_dir, &output_dir, silent)?;

    if watching {
        loop {
            watch(&src_dir, &output_dir, silent)?
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: draven -i <input_folder> -o <output_folder>");
    println!("-w:  Watches for file change in input folder");
    println!("-h: Display help message");
    println!("-o <folder>: location to write markdown files to");
    println!("-i <folder>: location to get rust project from");
    println!("-s: Silent mode");
    process::exit(0);
}

fn work<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &PathBuf,
    silent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;
    traverse_directory(&src_dir, output_dir)?;
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
                            return work(&src_dir, output_dir, silent);
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
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            traverse_directory(&path, output_dir)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                parse_and_convert_to_markdown(&path, output_dir)?
            }
        }
    }
    Ok(())
}

fn parse_and_convert_to_markdown<P: AsRef<Path>>(
    path: P,
    output_dir: &Path,
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
                if let syn::Type::Path(type_path) = &field.ty {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map(|ident| ident.to_string())
                        .unwrap_or_else(|| "unnamed_field".to_string());
                    let type_name = resolve_type_path(type_path, &import_map);
                    markdown.push_str(&format!("{} : [[{}]]\n", field_name, type_name));
                }
            }

            let output_file = output_dir.join(format!("{}.md", struct_name));
            let mut file = fs::File::create(output_file)?;
            file.write_all(markdown.as_bytes())?;
        }
    }
    Ok(())
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
        UseTree::Glob(_glob) => {
            // Skip glob imports
        }
        UseTree::Group(group) => {
            for tree in &group.items {
                parse_use_tree(tree, import_map, prefix.clone());
            }
        }
    }
}

fn resolve_type_path(type_path: &syn::TypePath, import_map: &HashMap<String, String>) -> String {
    let segments = &type_path.path.segments;
    if let Some(first_segment) = segments.first() {
        if let Some(full_path) = import_map.get(&first_segment.ident.to_string()) {
            return format!(
                "{}{}",
                full_path,
                segments
                    .iter()
                    .skip(1)
                    .map(|segment| format!("::{}", segment.ident))
                    .collect::<String>()
            );
        }
    }
    segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
