# draven

draven creates obsidian graph files from a rust project structs in real time 🌟

![rustc compiler output](https://i.postimg.cc/dDMb3kfV/examplepage.webp)

##### Install

```bash
cargo install draven
```

##### Usage

```bash
draven -w -i "path/to/rust_project_src" -o "path/to/an_obsidian_vault"
```

- `-i <folder>` location to get rust project from
- `-o <folder>` location to write markdown files to
- `-h` Display help message
- `-w` Watches for file change in input folder
- `-p` Enable linking primitive types in markdown files
- `-s` Silent mode");
