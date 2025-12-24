use emuforge_core::plugin::ppsspp::PpssppPlugin;
use emuforge_core::plugin::EmulatorPlugin;
use std::fs::File;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_ppsspp_plugin_config_generation() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    // 1. Create a dummy "emulator" binary
    let emu_path = root.join("ppsspp_mock");
    File::create(&emu_path).unwrap();

    // 2. Create a dummy ROM
    let rom_path = root.join("game.iso");
    File::create(&rom_path).unwrap();

    // 3. Instantiate plugin with override
    let plugin = PpssppPlugin::new(Some(emu_path.clone()));

    // 4. Generate Launch Config (Simulation)
    let config = plugin.prepare_launch_config(&rom_path, root).expect("Failed to prepare config");

    // 5. Assertions
    assert_eq!(config.emulator_path, emu_path);
    assert_eq!(config.rom_path, rom_path);
    
    // Note: Fullscreen is NOT added by default in the generic prepare_launch_config anymore 
    // unless logic inside plugin enforces it regardless of args.
    // In our implementation, we removed the mandatory push.
    // Wait, the plugin logic I wrote (modified) *commented out* the default push.
    // So args should be empty here.
    
    assert!(config.args.is_empty());
}
