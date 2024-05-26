# draven

draven creates obsidian graph files from a rust project structs in real time 🌟

![rustc compiler output](https://i.postimg.cc/dDMb3kfV/examplepage.webp)

##### Install

[obsidian](https://obsidian.md/) required

```rs
cargo install draven
```

##### Usage

```bash
draven -w -r -i "path/to/rust_project_src" -o "path/to/an_obsidian_vault"
```

- `-h` Display help message
- `-i <src_path>` location to get rust project src from
- `-o <vault_path>` location to write the obsidian files to
- `-w` Watches for file change in real time to update obsidian files
- `-r` Enable recursive folder processing in input folder
- `-p` Enable primitive types linking in markdown files
- `-s` Silent mode
