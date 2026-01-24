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
        if let Ok(path) = which::which("azahar-wayland") { return Ok(path); }
        if let Ok(path) = which::which("azahar-wayland.AppImage") { return Ok(path); }
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
        
        // Générer qt-config.ini avec DEUX profils:
        // - Profil 1 (index 0): Manette SDL (défaut si manette détectée)
        // - Profil 2 (index 1): Clavier (fallback si pas de manette)
        // Le stub modifiera dynamiquement "profile=X" avant le lancement
        let config_content = r#"[Controls]
profile=0
profile\default=true
profiles\1\name=Manette
        profiles\1\button_a="button:0,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_b="button:1,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_x="button:2,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_y="button:3,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_start="button:7,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_select="button:6,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_l="button:4,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_r="button:5,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_zl="axis:2,direction:+,engine:sdl,guid:030081b85e0400008e02000010010000,port:0,threshold:0.5"
        profiles\1\button_zr="axis:5,direction:+,engine:sdl,guid:030081b85e0400008e02000010010000,port:0,threshold:0.5"
        profiles\1\button_home="button:8,engine:sdl,guid:030081b85e0400008e02000010010000,port:0"
        profiles\1\button_up="direction:up,engine:sdl,guid:030081b85e0400008e02000010010000,hat:0,port:0"
        profiles\1\button_down="direction:down,engine:sdl,guid:030081b85e0400008e02000010010000,hat:0,port:0"
        profiles\1\button_left="direction:left,engine:sdl,guid:030081b85e0400008e02000010010000,hat:0,port:0"
        profiles\1\button_right="direction:right,engine:sdl,guid:030081b85e0400008e02000010010000,hat:0,port:0"
        profiles\1\button_debug="code:79,engine:keyboard"
        profiles\1\button_gpio14="code:80,engine:keyboard"
        profiles\1\button_power="code:86,engine:keyboard"
        profiles\1\circle_pad="down:axis$01$1direction$0+$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$00.5,engine:analog_from_button,left:axis$00$1direction$0-$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$0-0.5,modifier:code$068$1engine$0keyboard,modifier_scale:0.480000,right:axis$00$1direction$0+$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$00.5,up:axis$01$1direction$0-$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$0-0.5"
        profiles\1\c_stick="down:axis$04$1direction$0+$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$00.5,engine:analog_from_button,left:axis$03$1direction$0-$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$0-0.5,modifier:code$068$1engine$0keyboard,modifier_scale:0.500000,right:axis$03$1direction$0+$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$00.5,up:axis$04$1direction$0-$1engine$0sdl$1guid$0030081b85e0400008e02000010010000$1port$00$1threshold$0-0.5"
profiles\1\motion_device="engine:motion_emu,update_period:100,sensitivity:0.01,tilt_clamp:90.0"
profiles\1\touch_device=engine:emu_window
profiles\1\use_touch_from_button=false
profiles\1\touch_from_button_map=0
profiles\1\udp_input_address=127.0.0.1
profiles\1\udp_input_port=26760
profiles\1\udp_pad_index=0
profiles\2\name=Clavier
profiles\2\button_a="code:65,engine:keyboard"
profiles\2\button_b="code:83,engine:keyboard"
profiles\2\button_x="code:90,engine:keyboard"
profiles\2\button_y="code:88,engine:keyboard"
profiles\2\button_start="code:77,engine:keyboard"
profiles\2\button_select="code:78,engine:keyboard"
profiles\2\button_l="code:81,engine:keyboard"
profiles\2\button_r="code:87,engine:keyboard"
profiles\2\button_zl="code:49,engine:keyboard"
profiles\2\button_zr="code:50,engine:keyboard"
profiles\2\button_home="code:66,engine:keyboard"
profiles\2\button_up="code:84,engine:keyboard"
profiles\2\button_down="code:71,engine:keyboard"
profiles\2\button_left="code:70,engine:keyboard"
profiles\2\button_right="code:72,engine:keyboard"
profiles\2\button_debug="code:79,engine:keyboard"
profiles\2\button_gpio14="code:80,engine:keyboard"
profiles\2\button_power="code:86,engine:keyboard"
profiles\2\circle_pad="down:code$016777237$1engine$0keyboard,engine:analog_from_button,left:code$016777234$1engine$0keyboard,modifier:code$068$1engine$0keyboard,modifier_scale:0.500000,right:code$016777236$1engine$0keyboard,up:code$016777235$1engine$0keyboard"
profiles\2\c_stick="down:code$075$1engine$0keyboard,engine:analog_from_button,left:code$074$1engine$0keyboard,modifier:code$068$1engine$0keyboard,modifier_scale:0.500000,right:code$076$1engine$0keyboard,up:code$073$1engine$0keyboard"
profiles\2\motion_device="engine:motion_emu,update_period:100,sensitivity:0.01,tilt_clamp:90.0"
profiles\2\touch_device=engine:emu_window
profiles\2\use_touch_from_button=false
profiles\2\touch_from_button_map=0
profiles\2\udp_input_address=127.0.0.1
profiles\2\udp_input_port=26760
profiles\2\udp_pad_index=0
profiles\size=2
touch_from_button_maps\1\name=default
touch_from_button_maps\1\entries\size=0
touch_from_button_maps\size=1
use_artic_base_controller=false

[Core]
use_cpu_jit=true

[Renderer]
use_disk_shader_cache=true
use_hw_shader=true
shaders_accurate_mul=true
use_shader_jit=true
graphics_api=2
resolution_factor=3

[Layout]
custom_layout=false
factor_3d=15

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
        fs::write(&config_path, config_content).context("Failed to write qt-config.ini")?;
        
        Ok(())
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(AzaharPlugin::new(Some(binary_path)))
    }
}
