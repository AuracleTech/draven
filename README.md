# draven

draven parses structures of a rust project into obsidian vault graph files 🌟

![rustc compiler output](https://i.postimg.cc/dDMb3kfV/examplepage.webp)

##### Install

```bash
cargo install draven
```

##### Usage

- `-w` Watch for changes in input folder
- `-h` Display help message
- `-o` Output folder to write markdown files to
- `-i` Input folder to get rust project from
- `-s` Silent mode

```bash
draven -i "path/to/rust_project" -o "path/to/an_obsidian_vault"
```
