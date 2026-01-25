use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

/// Configuration PCSX2 par défaut pour éviter le Setup Wizard
const PCSX2_INI_CONTENT: &str = r#"[UI]
SettingsVersion = 1
InhibitScreensaver = true
ConfirmShutdown = true
StartPaused = false
PauseOnFocusLoss = false
StartFullscreen = true
DoubleClickTogglesFullscreen = true
HideMouseCursor = false
RenderToSeparateWindow = false
HideMainWindowWhenRunning = true
DisableWindowResize = false
PreferEnglishGameList = false
Theme = darkfusion
SetupWizardIncomplete = false
MainWindowGeometry = AdnQywADAAAAAAEyAAAAVgAABUsAAAMOAAABMgAAAHUAAAVLAAADDgAAAAAAAAAABn0AAAEyAAAAdQAABUsAAAMO
MainWindowState = AAAA/wAAAAD9AAAAAAAABBoAAAJvAAAABAAAAAQAAAAIAAAACPwAAAABAAAAAgAAAAEAAAAOAHQAbwBvAGwAQgBhAHIAAAAAAP////8AAAAAAAAAAA==

[Folders]
Bios = bios
Snapshots = snaps
Savestates = sstates
MemoryCards = memcards
Logs = logs
Cheats = cheats
Patches = patches
UserResources = resources
Cache = cache
Textures = textures
InputProfiles = inputprofiles
Videos = videos

[EmuCore]
CdvdVerboseReads = false
CdvdDumpBlocks = false
CdvdPrecache = false
EnablePatches = true
EnableCheats = false
EnablePINE = false
EnableWideScreenPatches = true
EnableNoInterlacingPatches = false
EnableFastBoot = true
EnableFastBootFastForward = false
EnableThreadPinning = false
EnableRecordingTools = true
EnableGameFixes = true
SaveStateOnShutdown = false
EnableDiscordPresence = false
InhibitScreensaver = true
HostFs = false
BackupSavestate = true
McdFolderAutoManage = true
WarnAboutUnsafeSettings = true
SavestateCompressionType = 2
SavestateCompressionRatio = 1
GzipIsoIndexTemplate = $(f).pindex.tmp
PINESlot = 28011
BlockDumpSaveDirectory = 

[EmuCore/Speedhacks]
EECycleRate = 0
EECycleSkip = 0
fastCDVD = false
IntcStat = true
WaitLoop = true
vuFlagHack = true
vuThread = true
vu1Instant = true

[EmuCore/CPU]
FPU.DenormalsAreZero = true
FPU.Roundmode = 3
FPUDiv.DenormalsAreZero = true
FPUDiv.Roundmode = 0
VU0.DenormalsAreZero = true
VU0.Roundmode = 3
VU1.DenormalsAreZero = true
VU1.Roundmode = 3
ExtraMemory = false

[EmuCore/CPU/Recompiler]
EnableEE = true
EnableIOP = true
EnableEECache = false
EnableVU0 = true
EnableVU1 = true
EnableFastmem = true
PauseOnTLBMiss = false
vu0Overflow = true
vu0ExtraOverflow = false
vu0SignOverflow = false
vu0Underflow = false
vu1Overflow = true
vu1ExtraOverflow = false
vu1SignOverflow = false
vu1Underflow = false
fpuOverflow = true
fpuExtraOverflow = false
fpuFullMode = false

[EmuCore/GS]
VsyncEnable = false
DisableMailboxPresentation = false
ExtendedUpscalingMultipliers = false
VsyncQueueSize = 2
FramerateNTSC = 59.94
FrameratePAL = 50
AspectRatio = Stretch
FMVAspectRatioSwitch = Off
ScreenshotSize = 0
ScreenshotFormat = 0
ScreenshotQuality = 50
StretchY = 100
CropLeft = 0
CropTop = 0
CropRight = 0
CropBottom = 0
pcrtc_antiblur = true
disable_interlace_offset = false
pcrtc_offsets = false
pcrtc_overscan = false
IntegerScaling = false
UseDebugDevice = false
UseBlitSwapChain = false
DisableShaderCache = false
DisableFramebufferFetch = false
DisableVertexShaderExpand = false
SkipDuplicateFrames = false
OsdShowSpeed = false
OsdShowFPS = false
OsdShowVPS = false
OsdShowCPU = false
OsdShowGPU = false
OsdShowResolution = false
OsdShowGSStats = false
OsdShowIndicators = true
OsdShowSettings = false
OsdShowInputs = false
OsdShowFrameTimes = false
OsdShowVersion = false
OsdShowHardwareInfo = false
OsdShowVideoCapture = true
OsdShowInputRec = true
HWSpinGPUForReadbacks = false
HWSpinCPUForReadbacks = false
paltex = false
autoflush_sw = true
preload_frame_with_gs_data = false
mipmap = true
UserHacks = false
UserHacks_align_sprite_X = false
UserHacks_AutoFlushLevel = 0
UserHacks_CPU_FB_Conversion = false
UserHacks_ReadTCOnClose = false
UserHacks_DisableDepthSupport = false
UserHacks_DisablePartialInvalidation = false
UserHacks_Disable_Safe_Features = false
UserHacks_DisableRenderFixes = false
UserHacks_merge_pp_sprite = false
UserHacks_ForceEvenSpritePosition = false
UserHacks_BilinearHack = 0
UserHacks_NativePaletteDraw = false
UserHacks_TextureInsideRt = 0
UserHacks_EstimateTextureRegion = false
fxaa = false
ShadeBoost = false
dump = false
save = false
savef = false
savet = false
savez = false
DumpReplaceableTextures = false
DumpReplaceableMipmaps = false
DumpTexturesWithFMVActive = false
DumpDirectTextures = true
DumpPaletteTextures = true
LoadTextureReplacements = false
LoadTextureReplacementsAsync = true
PrecacheTextureReplacements = false
EnableVideoCapture = true
EnableVideoCaptureParameters = false
VideoCaptureAutoResolution = true
EnableAudioCapture = true
EnableAudioCaptureParameters = false
linear_present_mode = 1
deinterlace_mode = 0
OsdScale = 100
OsdMessagesPos = 1
OsdPerformancePos = 2
Renderer = -1
upscale_multiplier = 1
hw_mipmap = true
accurate_blending_unit = 1
filter = 2
texture_preloading = 2
GSDumpCompression = 2
HWDownloadMode = 0
CASMode = 0
CASSharpness = 50
dithering_ps2 = 2
MaxAnisotropy = 0
extrathreads = 3
extrathreads_height = 4
TVShader = 0
UserHacks_TCOffsetX = 0
UserHacks_TCOffsetY = 0
TriFilter = -1
OverrideTextureBarriers = -1
ShadeBoost_Brightness = 50
ShadeBoost_Contrast = 50
ShadeBoost_Saturation = 50
ExclusiveFullscreenControl = -1
png_compression_level = 1
saven = 0
savel = 5000
CaptureContainer = mp4
VideoCaptureBitrate = 6000
VideoCaptureWidth = 640
VideoCaptureHeight = 480
AudioCaptureBitrate = 192
SyncToHostRefreshRate = false
UseVSyncForTiming = false

[InputSources]
Keyboard = true
Mouse = true
SDL = true
DInput = false
XInput = false
SDLControllerEnhancedMode = false
SDLPS5PlayerLED = false

[Pad]
MultitapPort1 = false
MultitapPort2 = false
PointerXScale = 8
PointerYScale = 8

[Pad1]
Type = DualShock2
InvertL = 0
InvertR = 0
Deadzone = 0
AxisScale = 1.33
LargeMotorScale = 1
SmallMotorScale = 1
ButtonDeadzone = 0
PressureModifier = 0.5
Up = Keyboard/Up
Up = SDL-0/DPadUp
Right = Keyboard/Right
Right = SDL-0/DPadRight
Down = Keyboard/Down
Down = SDL-0/DPadDown
Left = Keyboard/Left
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
Start = Keyboard/Return
Start = SDL-0/Start
L1 = Keyboard/Q
L1 = SDL-0/LeftShoulder
L2 = Keyboard/1
L2 = SDL-0/+LeftTrigger
R1 = Keyboard/E
R1 = SDL-0/RightShoulder
R2 = Keyboard/3
R2 = SDL-0/+RightTrigger
L3 = Keyboard/2
L3 = SDL-0/LeftStick
R3 = Keyboard/4
R3 = SDL-0/RightStick
LUp = Keyboard/W
LUp = SDL-0/-LeftY
LRight = Keyboard/D
LRight = SDL-0/+LeftX
LDown = Keyboard/S
LDown = SDL-0/+LeftY
LLeft = Keyboard/A
LLeft = SDL-0/-LeftX
RUp = Keyboard/T
RUp = SDL-0/-RightY
RRight = Keyboard/H
RRight = SDL-0/+RightX
RDown = Keyboard/G
RDown = SDL-0/+RightY
RLeft = Keyboard/F
RLeft = SDL-0/-RightX
Analog = SDL-0/Guide
LargeMotor = SDL-0/LargeMotor
SmallMotor = SDL-0/SmallMotor

[Pad2]
Type = None
"#;

pub struct Pcsx2Plugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl Pcsx2Plugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for Pcsx2Plugin {
    fn id(&self) -> &str { "pcsx2" }
    fn name(&self) -> &str { "PCSX2 (PS2 Emulator)" }
    fn supported_extensions(&self) -> &[&str] { &["iso", "cso", "bin", "gz", "chd"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        // Try standard paths via PATH
        if let Ok(path) = which::which("pcsx2-qt") { return Ok(path); }
        if let Ok(path) = which::which("pcsx2x64") { return Ok(path); }
        if let Ok(path) = which::which("pcsx2") { return Ok(path); }

        // Try common Windows installation paths
        #[cfg(target_os = "windows")]
        {
            let common_paths = [
                r"C:\Program Files\PCSX2\pcsx2-qt.exe",
                r"C:\Program Files (x86)\PCSX2\pcsx2-qt.exe",
                r"C:\PCSX2\pcsx2-qt.exe",
                r"C:\Program Files\PCSX2\pcsx2-qtx64-avx2.exe", 
                r"C:\Program Files (x86)\PCSX2\pcsx2-qtx64-avx2.exe",
                r"C:\PCSX2\pcsx2-qtx64-avx2.exe",
                r"C:\Program Files\PCSX2\pcsx2x64.exe",
                r"C:\Program Files (x86)\PCSX2\pcsx2x64.exe",
                r"C:\PCSX2\pcsx2x64.exe",
            ];

            for path_str in common_paths {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
            
            // Check LocalAppData
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data).join("PCSX2").join("pcsx2-qt.exe");
                if path.exists() { return Ok(path); }
            }
        }
        
        // EmuForge Managed / Recursive Home Check (Cross-platform)
        // Checks ~/.emuforge/emulators/pcsx2 recursively
        let emuforge_names = ["pcsx2-qt.exe", "pcsx2-qtx64.exe", "pcsx2x64.exe", "pcsx2.exe", "PCSX2.AppImage"];
        if let Some(path) = crate::plugin::find_in_emuforge_install("pcsx2", &emuforge_names) {
            return Ok(path);
        }
        
        Err(anyhow::anyhow!("PCSX2 executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate PCSX2 binary")?;
        
        // --- PORTABLE MODE STRATEGY ---
        // PCSX2 AppImage supports -portable flag, which creates all data alongside the binary.
        // We create a dedicated data folder and use -portable to isolate this game's config.
        
        let config_dir = output_dir.join("pcsx2_data");
        std::fs::create_dir_all(&config_dir).context("Failed to create config dir")?;
        
        // Convert to absolute path - critical for XDG_CONFIG_HOME to work
        let config_dir = config_dir.canonicalize().context("Failed to get absolute path for config dir")?;

        // --- WIN32 AUTO-PORTABLE MODE ---
        // On Windows, command line arguments like -cfgpath are unreliable with Qt versions.
        // The most robust method is to force portable mode by creating "portable.ini" next to the exe.
        // This makes PCSX2 look for configs in its own directory (pcsx2_data/PCSX2/inis usually).
        #[cfg(target_os = "windows")]
        {
            if let Some(binary_dir) = binary.parent() {
                let portable_ini = binary_dir.join("portable.ini");
                if !portable_ini.exists() {
                    eprintln!("⚙️  Creating portable.ini to force PCSX2 portable mode at: {:?}", portable_ini);
                    if let Err(e) = std::fs::File::create(&portable_ini) {
                        eprintln!("⚠️  Failed to create portable.ini: {}", e);
                    } else {
                        eprintln!("✅ portable.ini created successfully.");
                    }
                }
            }
        }


        // Create bios subfolder - must be inside PCSX2/ to match XDG structure
        // When XDG_CONFIG_HOME=pcsx2_data, PCSX2 looks for bios in pcsx2_data/PCSX2/bios
        let bios_dir = config_dir.join("PCSX2").join("bios");
        std::fs::create_dir_all(&bios_dir).context("Failed to create bios dir")?;

        // Generate minimal ini to skip the First Run Wizard
        // PCSX2 Qt with XDG_CONFIG_HOME looks for config in $XDG_CONFIG_HOME/PCSX2/inis/
        let ini_path = config_dir.join("PCSX2").join("inis").join("PCSX2.ini");
        std::fs::create_dir_all(ini_path.parent().unwrap())?;
        
        // The critical setting is SetupWizardIncomplete = false
        // We also need to point to the bios folder within our config structure
        // Include default keyboard bindings so the game responds to input
        std::fs::write(&ini_path, PCSX2_INI_CONTENT).context("Failed to write PCSX2.ini")?;

        // Note: SDL controller auto-mapping requires the PCSX2 Qt GUI to be used at least once.
        // Users will need to open PCSX2 normally and use "Automatic Mapping" for gamepads.
        // The launcher provides keyboard bindings that work out of the box.

        // Argument order matters: ROM is added first by the stub logic now.
        // -batch: Exits after game closes
        // -nogui: Hides main window
        // -fullscreen: Starts in fullscreen
        // on Windows, XDG_CONFIG_HOME is often ignored by Qt apps unless forced.
        // We explicitly pass -config to point to our directory.
        // Also note: PCSX2 Qt arguments regarding config paths can be tricky.
        // Using -portable might be better if we could, but we can't create 'portable.ini' next to the exe easily.
        
        let args = vec![
            "-batch".to_string(),
            "-nogui".to_string(),
            "-fullscreen".to_string(),
            // Remove -cfgpath as it causes "unknown parameter" errors on some Windows versions.
            // We rely on XDG_CONFIG_HOME / PCSX2_USER_PATH env vars set below.
            // "-cfgpath".to_string(),
            // config_dir.to_string_lossy().to_string(), 
        ];
        
        // Use environment variables to control config location as fallback
        let env_vars = vec![
            ("XDG_CONFIG_HOME".to_string(), config_dir.to_string_lossy().to_string()),
            ("PCSX2_USER_PATH".to_string(), config_dir.to_string_lossy().to_string()),
        ];
        
        // We store the bios_dir path in a special field so lib.rs can copy the BIOS file there
        // Actually, LaunchConfig doesn't have a bios_dir field. We can use working_dir or just
        // handle it differently. Let me reconsider.
        //
        // Alternative: Return the bios destination path via an environment variable or just
        // have lib.rs reconstruct it (output/pcsx2_data/bios/).
        //
        // Simpler: lib.rs will check if driver_id == "pcsx2", and if so, copy bios to
        // output_dir/pcsx2_data/bios/<filename>.
        //
        // For now, let's just return the config. The copying logic will be in lib.rs.

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            args,
            env_vars,
            ..Default::default()
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("pcsx2")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        // Déjà inclus dans prepare_launch_config (-fullscreen)
        vec![]
    }

    fn setup_environment(&self, _output_dir: &Path, bios_path: Option<&Path>) -> Result<()> {
        // WIN32 PORTABLE FIX:
        // Instead of writing to the temporary output_dir, we MUST write to the actual PCSX2 binary directory.
        // This is because we force "portable mode" with portable.ini, causing PCSX2 to ignore external configs.
        
        let binary_path = self.find_binary().context("Could not find PCSX2 binary to setup environment")?;
        let binary_dir = binary_path.parent().expect("PCSX2 binary has no parent dir");
        
        eprintln!("⚙️  Setting up PCSX2 environment in: {:?}", binary_dir);

        // 1. Create/Ensure 'bios' directory next to executable
        let bios_dest_dir = binary_dir.join("bios");
        std::fs::create_dir_all(&bios_dest_dir).context("Failed to create BIOS directory in emulator folder")?;
        
        // 2. Copy BIOS if provided
        if let Some(bios) = bios_path {
            eprintln!("🔍 BIOS provided: {:?}", bios);
            if bios.exists() {
                let bios_filename = bios.file_name().ok_or_else(|| anyhow::anyhow!("Invalid BIOS path"))?;
                let bios_dest = bios_dest_dir.join(bios_filename);
                eprintln!("📂 Copying BIOS to: {:?}", bios_dest);
                
                // Copy and overwrite to ensure we use the selected one
                std::fs::copy(bios, &bios_dest).context("Failed to copy BIOS file")?;
                eprintln!("✅ BIOS copied successfully!");
            } else {
                eprintln!("⚠️  BIOS file does not exist, skipping copy");
            }
        } else {
            eprintln!("⚠️  No BIOS provided");
        }
        
        // 3. Create 'inis' directory and write PCSX2.ini
        let inis_dir = binary_dir.join("inis");
        std::fs::create_dir_all(&inis_dir).context("Failed to create inis directory")?;
        
        let ini_path = inis_dir.join("PCSX2.ini");
        
        // Only write if it doesn't exist OR if we want to enforce EmuForge settings?
        // Enforcing is safer for "Plug and Play" to avoid broken user configs.
        eprintln!("📝 Writing config to: {:?}", ini_path);
        std::fs::write(&ini_path, PCSX2_INI_CONTENT).context("Failed to write PCSX2.ini")?;
        
        Ok(())
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(Pcsx2Plugin::new(Some(binary_path)))
    }

    fn portable_env_vars(&self, _config_dir: &Path) -> Vec<(String, String)> {
        // PCSX2 AppImage utilise souvent XDG_CONFIG_HOME
        // Structure attendue: $XDG_CONFIG_HOME/PCSX2/inis/PCSX2.ini
        // Notre config_dir est "pcsx2_data", qui contient le dossier "PCSX2".
        // FIX: Utiliser {exe_dir} pour que le Stub remplace par le dossier de l'executable au runtime
        vec![
            ("XDG_CONFIG_HOME".to_string(), "{exe_dir}/pcsx2_data".to_string()),
            ("PCSX2_USER_PATH".to_string(), "{exe_dir}/pcsx2_data".to_string()),
        ]
    }

    fn portable_launch_args(&self, fullscreen: bool) -> (Vec<String>, Vec<String>) {
        // PCSX2 syntax: [flags] [rom] (flags AVANT la ROM)
        let before = if fullscreen { 
            vec!["-fullscreen".to_string()] 
        } else { 
            vec![] 
        };
        (before, vec![])
    }
}
