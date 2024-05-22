fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_src_dir = "C:\\Users\\USER\\Desktop\\ash-master"; // path to the rust project
    let output_obsidian_vault = "C:\\Users\\USER\\Downloads\\ObsidianAsh"; // path to the your obsidian vault folder

    draven::structures_to_obsidian(input_src_dir, output_obsidian_vault)
}
