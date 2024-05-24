use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::PathBuf;
use std::{env, process};
use std::{fs, path::Path};
use syn::Item;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let (mut src_dir, mut output_dir) = (String::new(), String::new());
    let (mut watching, mut silent) = (false, false);

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "-help" | "--help" => print_help(),
            "-w" | "-watch" | "--watch" => watching = true,
            "-s" | "-silent" | "--silent" => silent = true,
            _ => Err(format!("Unknown argument: {}", arg))?,
    }

    if src_dir.is_empty() {
        eprintln!("Input folder required");
        eprintln!("Help: draven -help");
        process::exit(1);
    }

    if output_dir.is_empty() {
        eprintln!("Output folder required");
        eprintln!("Help: draven -help");
        process::exit(1);
    }

    let src_dir = PathBuf::from(src_dir);
    let mut output_dir = PathBuf::from(output_dir);

    output_dir.push("draven_generated");

    fs::create_dir_all(&output_dir)?;
    clean_markdown_in_directory(&output_dir)?;
    traverse_directory(&src_dir, &output_dir)?;
    if !silent {
        println!("Markdown files generated");
    }

    if watching {
        loop {
            if let Err(error) = watch(&src_dir, &output_dir, silent) {
                eprintln!("Error: {:?}", error);
                std::process::exit(1);
            }
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
        println!("Watching for changes in {:?}...", src_dir.as_ref());
    }

    for res in rx {
        match res {
            Ok(event) => {
                if let Some(path) = event.paths.first() {
                    if let Some(extension) = path.extension() {
                        if extension == "rs" {
                            if !silent {
                                println!("Regenerating markdown files...");
                            }
                            fs::create_dir_all(&output_dir)?;
                            clean_markdown_in_directory(&output_dir)?;
                            return traverse_directory(src_dir, &output_dir);
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("Error: {:?}", error);
                return Err(error.into());
            }
        }
    }

    Ok(())
}

fn clean_markdown_in_directory<P: AsRef<Path>>(
    output_dir: P,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "md" {
                    fs::remove_file(&path)?;
                }
            }
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
    output_dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(&path)?;
    let syntax_res = syn::parse_file(&content);
    if syntax_res.is_err() {
        return Ok(());
    }
    let syntax = syntax_res?;
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
                    let type_name = type_path
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_else(|| "unnamed_type".to_string());
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
