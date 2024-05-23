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

- `-w` Watches for file change in input folder
- `-h` Display help message
- `-o` Output folder to write markdown files to
- `-i` Input folder to get rust project from
- `-s` Silent mode
