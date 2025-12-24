use emuforge_core::forge::{ExecutableForge, LaunchConfig};
use std::path::PathBuf;
use std::io::Write;

fn main() -> anyhow::Result<()> {
    println!("🧪 Starting EmuForge Manual Test");

    // 1. Setup paths
    // We assume we are running from workspace root or crate root
    let root = PathBuf::from(".").canonicalize()?;
    println!("Root: {:?}", root);
    
    // Stub path: ../stub or stub depending on where we run
    let stub_path = if root.join("emuforge/stub/Cargo.toml").exists() {
        root.join("emuforge/stub")
    } else if root.join("stub/Cargo.toml").exists() {
        root.join("stub")
    } else {
        panic!("Cannot find stub crate!");
    };
    
    let output_dir = std::env::temp_dir().join("emuforge_test_out");
    if output_dir.exists() {
         std::fs::remove_dir_all(&output_dir)?;
    }
    
    // 2. Create dummy ROM
    let rom_path = std::env::temp_dir().join("test_rom.txt");
    let mut file = std::fs::File::create(&rom_path)?;
    writeln!(file, "I am a ROM")?;
    
    // 3. Config
    let config = LaunchConfig {
        emulator_path: PathBuf::from("/bin/cat"), // Use 'cat' to print content of ROM
        rom_path: rom_path.clone(),
        args: vec![],
        working_dir: None,
        env_vars: vec![],
    };
    
    // 4. Forge
    println!("🔨 Forging...");
    let forge = ExecutableForge::new(stub_path, output_dir.clone());
    
    let exe_path = forge.forge("TestGame", &config)?;
    println!("✅ Forged at: {:?}", exe_path);
    
    // 5. Run the forged executable
    println!("🚀 Running forged executable...");
    
    let status = std::process::Command::new(&exe_path)
        .status()?;
        
    if status.success() {
        println!("✅ Test Passed: Executable ran successfully");
    } else {
        println!("❌ Test Failed: Executable returned error");
    }
    
    Ok(())
}
