use anyhow::{Context, Result};
use std::path::PathBuf;

use std::process::Command;
use std::fs;
use crate::forge::LaunchConfig;

pub struct ExecutableForge {
    /// Path to the stub crate directory (containing Cargo.toml)
    pub stub_crate_path: PathBuf,
    /// Destination directory for forged executables
    pub output_dir: PathBuf,
}

impl ExecutableForge {
    pub fn new(stub_crate_path: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            stub_crate_path,
            output_dir,
        }
    }

    pub fn forge(&self, game_name: &str, config: &LaunchConfig) -> Result<PathBuf> {
        // 1. Create a build directory
        let build_dir = std::env::temp_dir().join("emuforge").join(game_name);
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).context("Failed to clean build dir")?;
        }
        fs::create_dir_all(&build_dir).context("Failed to create build dir")?;

        // 2. Serialize config
        let config_path = build_dir.join("launch_config.json");
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(&config_path, config_json).context("Failed to write config file")?;

        // 3. Compile the stub with the injected config
        // We use the 'stub' crate located at stub_crate_path
        let cargo_toml_path = self.stub_crate_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            anyhow::bail!("Stub Cargo.toml not found at {:?}", cargo_toml_path);
        }

        println!("Compiling stub for {}...", game_name);
        let output = Command::new("cargo")
            .args(&["build", "--release", "--manifest-path", cargo_toml_path.to_str().unwrap(), "--target-dir", build_dir.join("target").to_str().unwrap()])
            .env("EMUFORGE_CONFIG_PATH", &config_path)
            .output()
            .context("Failed to run cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Stub compilation failed: {}", stderr);
        }

        // 4. Locate the compiled binary
        // Note: With --target-dir, the binary is in <target-dir>/release/
        #[cfg(windows)]
        let binary_name = "emuforge-stub.exe";
        #[cfg(not(windows))]
        let binary_name = "emuforge-stub";

        let built_binary = build_dir.join("target").join("release").join(binary_name);
        
        if !built_binary.exists() {
             anyhow::bail!("Compiled binary not found at {:?}", built_binary);
        }

        // 5. Copy to output
        fs::create_dir_all(&self.output_dir)?;
        
        #[cfg(windows)]
        let final_name = format!("{}.exe", game_name);
        #[cfg(not(windows))]
        let final_name = game_name.to_string();

        let final_path = self.output_dir.join(&final_name);
        
        fs::copy(&built_binary, &final_path).context("Failed to copy binary to output")?;

        Ok(final_path)
    }
}
