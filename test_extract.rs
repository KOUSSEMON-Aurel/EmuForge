// Test script - run with: cargo run --example test_extract
use std::path::Path;
fn main() {
    let pup = "/media/aurel/THUSK_ACT_4/GAMES/BIOS/ps3/PS3UPDAT492-jan9.PUP";
    let out = "/home/aurel/Downloads/EmuForge_Output/test_extract";
    match emuforge_core::firmware::ps3::extract_firmware(Path::new(pup), Path::new(out)) {
        Ok(p) => println!("SUCCESS: {:?}", p),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
