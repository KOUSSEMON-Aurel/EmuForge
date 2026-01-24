use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct AzaharPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl AzaharPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for AzaharPlugin {
    fn id(&self) -> &str { "azahar" }
    fn name(&self) -> &str { "Azahar (3DS)" }
    fn supported_extensions(&self) -> &[&str] { &["3ds", "cia", "cxi", "cci", "3dsx"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("azahar") { return Ok(path); }
        if let Ok(path) = which::which("azahar-qt") { return Ok(path); }
        if let Ok(path) = which::which("lime3ds-cli") { return Ok(path); }
        if let Ok(path) = which::which("lime3ds-gui") { return Ok(path); }
        if let Ok(path) = which::which("lime3ds") { return Ok(path); }
        // Fallback for Citra users? Maybe later.
        
        Err(anyhow::anyhow!("Azahar executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Azahar binary")?;
        
        // Azahar Args: <rom>
        // No complicate args needed usually.
        let args = vec![];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            args_after_rom: vec![],
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("lime3ds") || name.contains("citra") || name.contains("azahar")
    }

    fn portable_env_vars(&self, config_dir: &Path) -> Vec<(String, String)> {
        // Rediriger la config vers le dossier local pour portabilité et injection
        vec![
            ("XDG_CONFIG_HOME".to_string(), config_dir.join("config").to_string_lossy().to_string()),
            ("XDG_DATA_HOME".to_string(), config_dir.join("data").to_string_lossy().to_string()),
        ]
    }

    fn setup_environment(&self, output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        use std::fs;
        
        let config_dir = output_dir.join("config/azahar-emu");
        fs::create_dir_all(&config_dir).context("Failed to create config dir")?;
        
        // Générer qt-config.ini avec mapping Manette + Clavier
        let config_content = r#"[Controls]
profile=0
profiles\1\name=Default
profiles\1\button_a="engine:sdl,joystick:0,button:1,engine:keyboard,code:65"
profiles\1\button_b="engine:sdl,joystick:0,button:0,engine:keyboard,code:83"
profiles\1\button_x="engine:sdl,joystick:0,button:3,engine:keyboard,code:90"
profiles\1\button_y="engine:sdl,joystick:0,button:2,engine:keyboard,code:88"
profiles\1\button_start="engine:sdl,joystick:0,button:7,engine:keyboard,code:77"
profiles\1\button_select="engine:sdl,joystick:0,button:6,engine:keyboard,code:78"
profiles\1\button_l="engine:sdl,joystick:0,button:4,engine:keyboard,code:81"
profiles\1\button_r="engine:sdl,joystick:0,button:5,engine:keyboard,code:87"
profiles\1\button_zl="engine:sdl,joystick:0,axis:2,direction:+,threshold:0.5,engine:keyboard,code:49"
profiles\1\button_zr="engine:sdl,joystick:0,axis:5,direction:+,threshold:0.5,engine:keyboard,code:50"
profiles\1\button_home="engine:sdl,joystick:0,button:8,engine:keyboard,code:66"
profiles\1\button_up="engine:sdl,joystick:0,hat:0,direction:up,engine:keyboard,code:84"
profiles\1\button_down="engine:sdl,joystick:0,hat:0,direction:down,engine:keyboard,code:71"
profiles\1\button_left="engine:sdl,joystick:0,hat:0,direction:left,engine:keyboard,code:70"
profiles\1\button_right="engine:sdl,joystick:0,hat:0,direction:right,engine:keyboard,code:72"
profiles\1\circle_pad="engine:sdl,joystick:0,axis_x:0,axis_y:1,engine:keyboard,up:code:84,down:code:71,left:code:70,right:72"
profiles\1\c_stick="engine:sdl,joystick:0,axis_x:3,axis_y:4,engine:keyboard,up:code:73,down:code:75,left:code:74,right:76"
profiles\size=1

[Core]
# Default core settings

[Renderer]
use_disk_shader_cache=true

[UI]
fullscreen=true
displayTitleBars=false
showFilterBar=false
showStatusBar=false
singleWindowMode=true
confirmClose=false
firstStart=false

[Data%20Storage]
use_virtual_sd=true
"#;

        let config_path = config_dir.join("qt-config.ini");
        if !config_path.exists() {
            fs::write(config_path, config_content).context("Failed to write qt-config.ini")?;
        }
        
        Ok(())
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(AzaharPlugin::new(Some(binary_path)))
    }
}
