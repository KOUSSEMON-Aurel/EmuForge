use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct MelonDSPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl MelonDSPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for MelonDSPlugin {
    fn id(&self) -> &str { "melonds" }
    fn name(&self) -> &str { "melonDS (NDS)" }
    fn supported_extensions(&self) -> &[&str] { &["nds", "srl", "dsi"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("melonDS") { return Ok(path); }
        if let Ok(path) = which::which("melonds") { return Ok(path); }
        if let Some(path) = &self.custom_binary_path {
             if path.exists() { return Ok(path.clone()); }
        }
        // Check for local AppImage if we renamed it
        if let Ok(path) = which::which("melonDS.AppImage") { return Ok(path); }
        
        Err(anyhow::anyhow!("melonDS executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate melonDS binary")?;
        
        // melonDS Args: <rom>
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
        name.contains("melonds")
    }

    fn portable_env_vars(&self, config_dir: &Path) -> Vec<(String, String)> {
        // Rediriger la config vers le dossier local pour portabilité
        vec![
            ("XDG_CONFIG_HOME".to_string(), config_dir.join("config").to_string_lossy().to_string()),
            ("XDG_DATA_HOME".to_string(), config_dir.join("data").to_string_lossy().to_string()),
        ]
    }

    fn setup_environment(&self, output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        use std::fs;
        
        // Structure config: {output_dir}/config/melonDS/melonDS.toml
        let config_dir = output_dir.join("config/melonDS");
        fs::create_dir_all(&config_dir).context("Failed to create melonDS config dir")?;
        
        // Configuration "Zero-Config" pour melonDS
        // - JoystickID = 0 (Premier contrôleur détecté)
        // - Mapping SDL standard (A=0, B=1, X=2, Y=3...)
        // - Clavier Standard
        // - Pas de GUID spécifique nécessaire ici, melonDS utilise les index SDL
        
        let config_content = r#"
LastBIOSFolder = ""
FastForwardFPS = 1000.0
PauseLostFocus = false
UITheme = ""
AudioSync = false
TargetFPS = 60.0
LimitFPS = true
SlowmoFPS = 30.0

[DS]
FirmwarePath = ""
BIOS7Path = ""
BIOS9Path = ""

[Instance0]
EnableCheats = false
SaveFilePath = ""
JoystickID = 0
SavestatePath = ""
CheatFilePath = ""

[Instance0.Window1]
Enabled = false

[Instance0.Keyboard]
Up = 16777235
HK_FrameLimitToggle = -1
HK_GuitarGripYellow = -1
HK_FastForwardToggle = -1
HK_SlowMoToggle = -1
HK_VolumeDown = -1
HK_GuitarGripBlue = -1
Y = 90
HK_FrameStep = -1
Down = 16777237
B = 83
A = 65
HK_SolarSensorIncrease = -1
X = 88
HK_GuitarGripRed = -1
L = 81
R = 87
HK_FullscreenToggle = -1
HK_SlowMo = -1
HK_Pause = -1
Right = 16777236
Start = -1
Select = -1
HK_Mic = -1
HK_FastForward = -1
Left = 16777234
HK_SwapScreens = -1
HK_SwapScreenEmphasis = -1
HK_PowerButton = -1
HK_Reset = -1
HK_Lid = -1
HK_SolarSensorDecrease = -1
HK_VolumeUp = -1
HK_GuitarGripGreen = -1

[Instance0.Joystick]
Up = 257
HK_FrameLimitToggle = -1
HK_GuitarGripYellow = -1
HK_FastForwardToggle = -1
HK_SlowMoToggle = -1
HK_VolumeDown = -1
HK_GuitarGripBlue = -1
Y = 3
HK_FrameStep = -1
Down = 260
B = 1
A = 0
HK_SolarSensorIncrease = -1
X = 2
HK_GuitarGripRed = -1
L = 4
R = 5
HK_FullscreenToggle = -1
HK_SlowMo = -1
HK_Pause = -1
Right = 258
Start = 7
Select = 6
HK_Mic = -1
HK_FastForward = -1
Left = 264
HK_SwapScreens = -1
HK_SwapScreenEmphasis = -1
HK_PowerButton = -1
HK_Reset = -1
HK_Lid = -1
HK_SolarSensorDecrease = -1
HK_VolumeUp = -1
HK_GuitarGripGreen = -1

[Instance0.Window0]
ShowOSD = true
ScreenLayout = 0
ScreenRotation = 0
ScreenAspectBot = 0
ScreenGap = 0
ScreenSwap = false
ScreenSizing = 0
IntegerScaling = false
ScreenAspectTop = 0
ScreenFilter = false
Enabled = true

[Instance0.Audio]
Volume = 256
DSiVolumeSync = false

[Instance0.DS]
[Instance0.DS.Battery]
LevelOkay = true

[3D]
Renderer = 2 # OpenGL

[3D.GL]
HiresCoordinates = true
BetterPolygons = false
ScaleFactor = 5

[3D.Soft]
Threaded = true

[Screen]
VSyncInterval = 1
VSync = true
UseGL = false
"#;

        let config_path = config_dir.join("melonDS.toml");
        fs::write(&config_path, config_content).context("Failed to write melonDS.toml")?;
        
        Ok(())
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(MelonDSPlugin::new(Some(binary_path)))
    }
}
