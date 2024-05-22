use std::io::Write;
use std::{env, process};
use std::{fs, path::Path};
use syn::Item;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        panic!("Usage: {} -o OUTPUT_FOLDER -i INPUT_FOLDER", args[0]);
    }

    let mut output = String::new();
    let mut input = String::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-o" => {
                if let Some(folder) = iter.next() {
                    output = folder.clone();
                } else {
                    eprintln!("Expected argument after -o");
                    process::exit(1);
                }
            }
            "-i" => {
                if let Some(folder) = iter.next() {
                    input = folder.clone();
                } else {
                    eprintln!("Expected argument after -i");
                    process::exit(1);
                }
            }
            _ => {}
        }
    }

    fs::create_dir_all(&output)?;
    traverse_directory(input, &output)
}

fn traverse_directory<P: AsRef<Path>>(
    src_dir: P,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            traverse_directory(&path, output_dir)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                parse_and_convert_to_markdown(&path, output_dir);
            }
        }
    }
    Ok(())
}

fn parse_and_convert_to_markdown<P: AsRef<Path>>(path: P, output_dir: &str) {
    let content = fs::read_to_string(&path).unwrap();
    let syntax_res = syn::parse_file(&content);
    if syntax_res.is_err() {
        return;
    }
    let syntax = syntax_res.unwrap();
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
                    let type_name = type_path.path.segments.last().unwrap().ident.to_string();
                    markdown.push_str(&format!("{} : [[{}]]\n", field_name, type_name));
                }
            }

            let file_path = format!("{}/{}.md", output_dir, struct_name);
            let mut file = fs::File::create(file_path).unwrap();
            file.write_all(markdown.as_bytes()).unwrap();
        }
    }
}
