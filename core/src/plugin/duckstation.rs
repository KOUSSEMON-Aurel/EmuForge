use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

/// Configuration complète pour DuckStation (PS1).
/// DuckStation ignore les variables XDG, donc on doit créer un faux HOME
/// avec la structure .local/share/duckstation/ et un wrapper script.
const DUCKSTATION_SETTINGS: &str = r#"[Main]
SettingsVersion = 3
EmulationSpeed = 1
FastForwardSpeed = 0
TurboSpeed = 0
SyncToHostRefreshRate = false
InhibitScreensaver = true
PauseOnFocusLoss = false
PauseOnControllerDisconnection = false
SaveStateOnExit = true
CreateSaveStateBackups = true
SaveStateCompression = ZstDefault
ConfirmPowerOff = true
EnableDiscordPresence = false
LoadDevicesFromSaveStates = false
DisableAllEnhancements = false
RewindEnable = false
RewindFrequency = 10
RewindSaveSlots = 10
RunaheadFrameCount = 0
RunaheadForAnalogInput = false
StartPaused = false
StartFullscreen = true
SetupWizardIncomplete = false

[Console]
Region = Auto
Enable8MBRAM = false

[CPU]
ExecutionMode = Recompiler
OverclockEnable = false
OverclockNumerator = 1
OverclockDenominator = 1
RecompilerMemoryExceptions = false
RecompilerBlockLinking = true
RecompilerICache = false
FastmemMode = MMap

[GPU]
Renderer = Automatic
Adapter = 
ResolutionScale = 0
Multisamples = 1
UseDebugDevice = false
UseGPUBasedValidation = false
PreferGLESContext = false
DisableShaderCache = false
DisableDualSourceBlend = false
DisableFramebufferFetch = false
DisableTextureBuffers = false
DisableTextureCopyToSelf = false
DisableMemoryImport = false
DisableRasterOrderViews = false
DisableComputeShaders = false
DisableCompressedTextures = false
PerSampleShading = false
MaxQueuedFrames = 2
UseThread = true
UseSoftwareRendererForReadbacks = false
UseSoftwareRendererForMemoryStates = false
ScaledInterlacing = true
ForceRoundTextureCoordinates = false
TextureFilter = Nearest
SpriteTextureFilter = Nearest
DitheringMode = TrueColor
LineDetectMode = Disabled
DownsampleMode = Disabled
DownsampleScale = 1
WireframeMode = Disabled
ForceVideoTiming = Disabled
WidescreenHack = false
EnableTextureCache = false
ChromaSmoothing24Bit = false
PGXPEnable = true
PGXPCulling = true
PGXPTextureCorrection = true
PGXPColorCorrection = false
PGXPVertexCache = false
PGXPCPU = false
PGXPPreserveProjFP = false
PGXPTolerance = -1
PGXPDepthBuffer = false
PGXPDisableOn2DPolygons = false
PGXPTransparentDepthTest = false
PGXPDepthThreshold = 4096
DumpFastReplayMode = false
DeinterlacingMode = Progressive

[Debug]
ShowVRAM = false
DumpCPUToVRAMCopies = false
DumpVRAMToCPUCopies = false
EnableGDBServer = false
GDBServerPort = 2345

[Display]
CropMode = Overscan
ActiveStartOffset = 0
ActiveEndOffset = 0
LineStartOffset = 0
LineEndOffset = 0
Force4_3For24Bit = false
AspectRatio = Stretch To Fill
FineCropMode = None
FineCropLeft = 0
FineCropTop = 0
FineCropRight = 0
FineCropBottom = 0
Alignment = Center
Rotation = Normal
Scaling = BilinearSmooth
Scaling24Bit = BilinearSmooth
OptimalFramePacing = false
PreFrameSleep = false
SkipPresentingDuplicateFrames = false
PreFrameSleepBuffer = 2
VSync = false
DisableMailboxPresentation = false
ExclusiveFullscreenControl = Automatic
ScreenshotMode = ScreenResolution
ScreenshotFormat = PNG
ScreenshotQuality = 85
ShowOSDMessages = true
ShowFPS = false
ShowSpeed = false
ShowResolution = false
ShowLatencyStatistics = false
ShowGPUStatistics = false
ShowCPU = false
ShowGPU = false
ShowFrameTimes = false
ShowStatusIndicators = true
ShowInputs = false
ShowEnhancements = false
OSDScale = 100
OSDMargin = 10
OSDErrorDuration = 15
OSDWarningDuration = 10
OSDInfoDuration = 5
OSDQuickDuration = 2.5
OSDPersistentDuration = 3.40282e+38
AutoResizeWindow = false

[CDROM]
ReadaheadSectors = 8
MechaconVersion = VC1A
RegionCheck = false
SubQSkew = false
LoadImageToRAM = false
LoadImagePatches = false
IgnoreHostSubcode = false
MuteCDAudio = false
AutoDiscChange = false
ReadSpeedup = 1
SeekSpeedup = 1
MaxReadSpeedupCycles = 30000
MaxSeekSpeedupCycles = 30000
DisableSpeedupOnMDEC = false

[Audio]
Backend = Cubeb
Driver = 
OutputDevice = 
StretchMode = TimeStretch
BufferMS = 50
OutputLatencyMS = 20
OutputLatencyMinimal = false
StretchSequenceLengthMS = 30
StretchSeekWindowMS = 20
StretchOverlapMS = 10
StretchUseQuickSeek = false
StretchUseAAFilter = false
OutputVolume = 100
FastForwardVolume = 100
OutputMuted = false

[Hacks]
UseOldMDECRoutines = false
ExportSharedMemory = false
DMAMaxSliceTicks = 1000
DMAHaltTicks = 100
GPUFIFOSize = 16
GPUMaxRunAhead = 128

[BIOS]
TTYLogging = false
PatchFastBoot = false
FastForwardBoot = false
SearchDirectory = bios

[MemoryCards]
Card1Type = PerGameTitle
Card2Type = None
UsePlaylistTitle = true
FastForwardAccess = false
Directory = memcards

[ControllerPorts]
MultitapMode = Disabled
PointerXScale = 8
PointerYScale = 8
PointerXInvert = false
PointerYInvert = false

[Cheevos]
Enabled = false
ChallengeMode = false
EncoreMode = false
SpectatorMode = false
UnofficialTestMode = false
UseRAIntegration = false
Notifications = true
LeaderboardNotifications = true
LeaderboardTrackers = true
SoundEffects = true
ProgressIndicators = true
ChallengeIndicatorMode = Notification
NotificationsDuration = 5
LeaderboardsDuration = 10

[TextureReplacements]
EnableTextureReplacements = false
EnableVRAMWriteReplacements = false
AlwaysTrackUploads = false
PreloadTextures = false
DumpVRAMWrites = false
DumpTextures = false
DumpReplacedTextures = true
DumpTexturePages = false
DumpFullTexturePages = false
DumpTextureForceAlphaChannel = false
DumpVRAMWriteForceAlphaChannel = true
DumpC16Textures = false
ReducePaletteRange = true
ConvertCopiesToWrites = false
ReplacementScaleLinearFilter = false
MaxHashCacheEntries = 1200
MaxHashCacheVRAMUsageMB = 2048
MaxReplacementCacheVRAMUsage = 512
MaxVRAMWriteSplits = 0
MaxVRAMWriteCoalesceWidth = 0
DumpTextureWidthThreshold = 16
DumpTextureHeightThreshold = 16
DumpVRAMWriteWidthThreshold = 128
DumpVRAMWriteHeightThreshold = 128

[PIO]
DeviceType = None
FlashImagePath = 
FlashImageWriteEnable = false
SwitchActive = true

[SIO]
RedirectToTTY = false

[PCDrv]
Enabled = false
EnableWrites = false
Root = 

[Logging]
LogLevel = Info
LogTimestamps = true
LogToConsole = false
LogToDebug = false
LogToWindow = false
LogToFile = false

[Folders]
Cache = cache
Cheats = cheats
Covers = covers
GameIcons = gameicons
GameSettings = gamesettings
InputProfiles = inputprofiles
Patches = patches
SaveStates = savestates
Screenshots = screenshots
Shaders = shaders
Subchannels = subchannels
Textures = textures
UserResources = resources
Videos = videos

[InputSources]
SDL = true
SDLControllerEnhancedMode = false
SDLPS5PlayerLED = false
XInput = false
RawInput = false

[Pad1]
Type = AnalogController
Up = Keyboard/UpArrow
Up = SDL-0/DPadUp
Right = Keyboard/RightArrow
Right = SDL-0/DPadRight
Down = Keyboard/DownArrow
Down = SDL-0/DPadDown
Left = Keyboard/LeftArrow
Left = SDL-0/DPadLeft
Triangle = Keyboard/I
Triangle = SDL-0/Y
Circle = Keyboard/L
Circle = SDL-0/B
Cross = Keyboard/K
Cross = SDL-0/A
Square = Keyboard/J
Square = SDL-0/X
Select = Keyboard/Backspace
Select = SDL-0/Back
Start = Keyboard/Enter
Start = SDL-0/Start
L1 = Keyboard/Q
L1 = SDL-0/LeftShoulder
R1 = Keyboard/E
R1 = SDL-0/RightShoulder
L2 = Keyboard/1
L2 = SDL-0/+LeftTrigger
R2 = Keyboard/3
R2 = SDL-0/+RightTrigger
L3 = Keyboard/2
L3 = SDL-0/LeftStick
R3 = Keyboard/4
R3 = SDL-0/RightStick
LLeft = Keyboard/A
LLeft = SDL-0/-LeftX
LRight = Keyboard/D
LRight = SDL-0/+LeftX
LDown = Keyboard/S
LDown = SDL-0/+LeftY
LUp = Keyboard/W
LUp = SDL-0/-LeftY
RLeft = Keyboard/F
RLeft = SDL-0/-RightX
RRight = Keyboard/H
RRight = SDL-0/+RightX
RDown = Keyboard/G
RDown = SDL-0/+RightY
RUp = Keyboard/T
RUp = SDL-0/-RightY
Analog = SDL-0/Guide
LargeMotor = SDL-0/LargeMotor
SmallMotor = SDL-0/SmallMotor

[Pad2]
Type = AnalogController
Up = SDL-1/DPadUp
Right = SDL-1/DPadRight
Down = SDL-1/DPadDown
Left = SDL-1/DPadLeft
Triangle = SDL-1/Y
Circle = SDL-1/B
Cross = SDL-1/A
Square = SDL-1/X
Select = SDL-1/Back
Start = SDL-1/Start
L1 = SDL-1/LeftShoulder
R1 = SDL-1/RightShoulder
L2 = SDL-1/+LeftTrigger
R2 = SDL-1/+RightTrigger
L3 = SDL-1/LeftStick
R3 = SDL-1/RightStick
LLeft = SDL-1/-LeftX
LRight = SDL-1/+LeftX
LDown = SDL-1/+LeftY
LUp = SDL-1/-LeftY
RLeft = SDL-1/-RightX
RRight = SDL-1/+RightX
RDown = SDL-1/+RightY
RUp = SDL-1/-RightY
Analog = SDL-1/Guide
LargeMotor = SDL-1/LargeMotor
SmallMotor = SDL-1/SmallMotor

[Pad3]
Type = None

[Pad4]
Type = None

[Pad5]
Type = None

[Pad6]
Type = None

[Pad7]
Type = None

[Pad8]
Type = None

[Hotkeys]
FastForward = Keyboard/Tab
TogglePause = Keyboard/Space
Screenshot = Keyboard/F10
ToggleFullscreen = Keyboard/F11
OpenPauseMenu = Keyboard/Escape
LoadSelectedSaveState = Keyboard/F1
SaveSelectedSaveState = Keyboard/F2
SelectPreviousSaveStateSlot = Keyboard/F3
SelectNextSaveStateSlot = Keyboard/F4

[UI]
MainWindowX = 13
MainWindowY = 13
MainWindowWidth = 934
MainWindowHeight = 514
ShowGameList = false
ShowStartWizard = false
SetupWizardIncomplete = false

[AutoUpdater]
CheckAtStartup = false

[GameList]
RecursivePaths = {{EXE_DIR}}
"#;

pub struct DuckStationPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl DuckStationPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for DuckStationPlugin {
    fn id(&self) -> &str { "duckstation" }
    fn name(&self) -> &str { "DuckStation (PS1)" }
    fn supported_extensions(&self) -> &[&str] { &["bin", "cue", "iso", "chd", "m3u", "pbp"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("duckstation-qt-x64-ReleaseLTCG") { return Ok(path); }
        if let Ok(path) = which::which("duckstation-no-gui") { return Ok(path); }
        if let Ok(path) = which::which("duckstation") { return Ok(path); }
        
        Err(anyhow::anyhow!("DuckStation executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate DuckStation binary")?;
        
        // DuckStation CLI Args: [flags] -- <filename>
        // Note: Le wrapper script ajoutera -fullscreen et le rom_path
        let args = vec![
            "-fullscreen".to_string(),
            "--".to_string(),
        ];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("duckstation")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        // Géré par le wrapper script, pas besoin d'ajouter ici
        vec![]
    }

    fn requires_wrapper(&self) -> bool {
        // DuckStation ignore XDG_*, donc on DOIT utiliser un wrapper qui change HOME
        true
    }

    fn setup_environment(&self, output_dir: &Path, bios_path: Option<&Path>) -> Result<()> {
        // Créer la structure .duckstation_home/.local/share/duckstation/
        let fake_home = output_dir.join(".duckstation_home");
        let ds_data_dir = fake_home.join(".local/share/duckstation");
        std::fs::create_dir_all(&ds_data_dir)
            .context("Failed to create DuckStation data directory")?;

        // Écrire le settings.ini
        std::fs::write(ds_data_dir.join("settings.ini"), DUCKSTATION_SETTINGS)
            .context("Failed to write DuckStation settings.ini")?;

        // Copier le BIOS si fourni
        if let Some(bios) = bios_path {
            if bios.exists() {
                let bios_dest_dir = ds_data_dir.join("bios");
                std::fs::create_dir_all(&bios_dest_dir)
                    .context("Failed to create BIOS directory")?;
                
                if let Some(filename) = bios.file_name() {
                    std::fs::copy(bios, bios_dest_dir.join(filename))
                        .context("Failed to copy BIOS file")?;
                }
            }
        }

        Ok(())
    }

    fn generate_wrapper_script(
        &self,
        config: &LaunchConfig,
        output_dir: &Path,
        game_name: &str,
    ) -> Result<Option<PathBuf>> {
        let wrapper_script = format!(r#"#!/bin/bash
# EmuForge DuckStation Wrapper - Généré automatiquement
# DuckStation ignore les variables XDG, donc on change HOME

# Répertoire du script
SCRIPT_DIR="$( cd "$( dirname "${{BASH_SOURCE[0]}}" )" && pwd )"

# Chemin de l'émulateur (absolu depuis le build, ou local si portable)
BUILD_TIME_PATH="{}"
EMU_FILENAME=$(basename "$BUILD_TIME_PATH")
LOCAL_EMU="$SCRIPT_DIR/$EMU_FILENAME"

if [ -f "$LOCAL_EMU" ]; then
    REAL_EMULATOR="$LOCAL_EMU"
else
    REAL_EMULATOR="$BUILD_TIME_PATH"
fi

# Fake HOME pour isoler DuckStation
FAKE_HOME="$SCRIPT_DIR/.duckstation_home"

# Lancement avec HOME modifié
export HOME="$FAKE_HOME"
export QT_QPA_PLATFORM=xcb

# Lancer DuckStation
"$REAL_EMULATOR" -fullscreen -- "{}"
"#, config.emulator_path.display(), config.rom_path.display());

        let wrapper_path = output_dir.join(format!("{}_launcher.sh", game_name));
        std::fs::write(&wrapper_path, wrapper_script)
            .context("Failed to write wrapper script")?;

        // Rendre exécutable sur Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&wrapper_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&wrapper_path, perms)?;
        }

        Ok(Some(wrapper_path))
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(DuckStationPlugin::new(Some(binary_path)))
    }

    fn portable_env_vars(&self, config_dir: &Path) -> Vec<(String, String)> {
        // DuckStation ignore XDG, il faut définir HOME
        vec![
            ("HOME".to_string(), config_dir.to_string_lossy().to_string()),
            ("QT_QPA_PLATFORM".to_string(), "xcb".to_string()),
        ]
    }

    fn portable_launch_args(&self, fullscreen: bool) -> (Vec<String>, Vec<String>) {
        // DuckStation syntax: [flags] -- <file>
        let before = if fullscreen { 
            vec!["-fullscreen".to_string()] 
        } else { 
            vec![] 
        };
        // Le "--" sera ajouté et la ROM ensuite par le stub
        (before, vec![])
    }
}
