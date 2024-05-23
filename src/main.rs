use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::{env, process};
use std::{fs, path::Path};
use syn::Item;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let args = &args[1..];

    let mut output = String::new();
    let mut input = String::new();
    let mut watching = false;
    let mut silent = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" | "-input" | "--input" => {
                if let Some(folder) = iter.next() {
                    input = verify_folder(folder.clone());
                } else {
                    eprintln!("Expected argument after -i");
                    process::exit(1);
                }
            }
            "-o" | "-output" | "--output" => {
                if let Some(folder) = iter.next() {
                    output = verify_folder(folder.clone());
                } else {
                    eprintln!("Expected argument after -o");
                    process::exit(1);
                }
            }
            "-h" | "-help" | "--help" => {
                eprintln!("Usage: draven -i <input_folder> -o <output_folder>");
                eprintln!("-w: Watch for changes in input folder");
                eprintln!("-h: Display this help message");
                eprintln!("-o: Output folder to write markdown files");
                eprintln!("-i: Input folder to watch");
                eprintln!("-s: Silent mode");
                process::exit(1);
            }
            "-w" | "-watch" | "--watch" => watching = true,
            "-s" | "-silent" | "--silent" => silent = true,
            _ => {
                eprintln!("Unknown argument: {}", arg);
                process::exit(1);
            }
        }
    }

    if input.is_empty() {
        eprintln!("Input folder required");
        eprintln!("Help: draven -help");
        process::exit(1);
    }

    if output.is_empty() {
        eprintln!("Output folder required");
        eprintln!("Help: draven -help");
        process::exit(1);
    }

    if watching {
        loop {
            if let Err(error) = watch(&input, &output, silent) {
                eprintln!("Error: {:?}", error);
                std::process::exit(1);
            }
        }
    } else {
        fs::create_dir_all(&output)?;
        clean_markdown_in_directory(&output)?;
        traverse_directory(input, &output)?;
        if !silent {
            println!("Markdown files generated");
        }
        Ok(())
    }
}

fn verify_folder(folder: String) -> String {
    let path = Path::new(&folder);
    if !path.exists() {
        eprintln!("Folder {:?} does not exist", path);
        process::exit(1);
    }
    if !path.is_dir() {
        eprintln!("{:?} is not a folder", path);
        process::exit(1);
    }
    folder
}

fn watch<P: AsRef<Path>>(
    input: P,
    output: &str,
    silent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(input.as_ref(), RecursiveMode::Recursive)?;

    if !silent {
        println!("Watching for changes in {:?}...", input.as_ref());
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
                            fs::create_dir_all(&output)?;
                            clean_markdown_in_directory(&output)?;
                            return traverse_directory(input, &output);
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
        if path.is_dir() {
            clean_markdown_in_directory(&path)?;
        } else if let Some(extension) = path.extension() {
            if extension == "md" {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

fn traverse_directory<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &str,
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
    output_dir: &str,
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
                    // TEST let type_name = type_path.path.segments.last().unwrap().ident.to_string(); // Replace unwrap
                    let type_name = type_path
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_else(|| "unnamed_type".to_string());
                    markdown.push_str(&format!("{} : [[{}]]\n", field_name, type_name));
                }
            }

            let file_path = format!("{}/{}.md", output_dir, struct_name);
            let mut file = fs::File::create(file_path)?;
            file.write_all(markdown.as_bytes())?;
        }
    }
    Ok(())
}
