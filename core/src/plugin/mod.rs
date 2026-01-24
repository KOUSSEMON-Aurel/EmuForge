use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::forge::LaunchConfig;
use serde::{Deserialize, Serialize}; // Need serde for struct

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequirementInfo {
    pub needs_bios: bool,
    pub needs_firmware: bool,
    pub keys_file: Option<String>, // e.g., "prod.keys"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
    pub fixed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostSpecs {
    pub screen_width: u32,
    pub screen_height: u32,
    pub vulkan_support: bool,
}

pub trait EmulatorPlugin: Send + Sync {
    /// Unique identifier for the emulator (e.g., "ppsspp").
    fn id(&self) -> &str;
    
    /// User-friendly name of the emulator.
    fn name(&self) -> &str;
    
    /// List of file extensions supported by this emulator (without dot).
    fn supported_extensions(&self) -> &[&str];
    
    /// Locate the emulator binary on the host system.
    fn find_binary(&self) -> Result<PathBuf>;
    
    /// Prepare the launch configuration for a specific ROM.
    fn prepare_launch_config(&self, rom_path: &Path, output_dir: &Path) -> Result<LaunchConfig>;

    /// Check if the provided emulator binary matches this plugin.
    fn can_handle(&self, binary_path: &Path) -> bool;

    // =========================================
    // Nouvelles méthodes avec implémentation par défaut
    // =========================================

    /// Arguments pour activer le mode plein écran (ajoutés AVANT la ROM).
    /// Implémentation par défaut : --fullscreen (standard pour la plupart des émulateurs).
    fn fullscreen_args(&self) -> Vec<String> {
        vec!["--fullscreen".to_string()]
    }

    /// Arguments fullscreen à ajouter APRÈS la ROM.
    /// Par défaut vide. Override pour émulateurs comme Cemu qui requièrent -f après le jeu.
    fn fullscreen_args_after_rom(&self) -> Vec<String> {
        vec![]
    }

    /// Prépare l'environnement de l'émulateur (config, BIOS, etc.).
    /// Appelé avant le lancement pour créer les fichiers nécessaires.
    fn setup_environment(&self, _output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        Ok(())
    }

    /// Version avec rapport de progression (Optionnel).
    /// Par défaut appelle setup_environment sans progression.
    fn setup_environment_with_progress(
        &self,
        output_dir: &Path,
        bios_path: Option<&Path>,
        _progress: Option<&dyn Fn(String)>,
    ) -> Result<()> {
        self.setup_environment(output_dir, bios_path)
    }

    /// Version avec rapport de progression pour prepare_launch_config.
    /// Par défaut appelle prepare_launch_config.
    fn prepare_launch_config_with_progress(
        &self, 
        rom_path: &Path, 
        output_dir: &Path, 
        _progress: Option<&dyn Fn(String)>
    ) -> Result<LaunchConfig> {
        self.prepare_launch_config(rom_path, output_dir)
    }

    /// Version avec spécifications de l'hôte (résolution, vulkan, etc.)
    /// Par défaut ignore les specs et appelle prepare_launch_config_with_progress.
    fn prepare_launch_config_with_specs(
        &self,
        rom_path: &Path,
        output_dir: &Path,
        _host_specs: Option<HostSpecs>,
        progress: Option<&dyn Fn(String)>
    ) -> Result<LaunchConfig> {
        self.prepare_launch_config_with_progress(rom_path, output_dir, progress)
    }

    /// Indique si l'émulateur nécessite un wrapper script (ex: DuckStation qui ignore XDG).
    fn requires_wrapper(&self) -> bool {
        false
    }

    /// Génère le wrapper script si nécessaire.
    /// Retourne le chemin du script créé, ou None si pas de wrapper.
    fn generate_wrapper_script(
        &self,
        _config: &LaunchConfig,
        _output_dir: &Path,
        _game_name: &str,
    ) -> Result<Option<PathBuf>> {
        Ok(None)
    }

    /// Crée une nouvelle instance du plugin avec un chemin binaire personnalisé.
    /// Utilisé par PluginManager pour éviter le match/case répétitif.
    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin>;

    // =========================================
    // Méthodes pour le mode portable (stub)
    // =========================================

    /// Variables d'environnement spécifiques pour le mode portable.
    /// Le stub appliquera ces variables avant de lancer l'émulateur.
    /// `config_dir` est le chemin vers le dossier de config extrait.
    fn portable_env_vars(&self, config_dir: &Path) -> Vec<(String, String)> {
        let _ = config_dir;
        vec![]
    }

    /// Arguments de lancement pour le mode portable.
    /// Retourne (args_before_rom, args_after_rom) pour une flexibilité maximale.
    /// Le stub construira: [emulator] [args_before] [rom] [args_after]
    fn portable_launch_args(&self, fullscreen: bool) -> (Vec<String>, Vec<String>) {
        let before = if fullscreen { vec!["--fullscreen".to_string()] } else { vec![] };
        (before, vec![])
    }
    
    /// Prépare le binaire de l'émulateur pour mode portable (patch si nécessaire)
    /// Retourne Some(PathBuf) si un binaire patché a été créé, None sinon
    fn prepare_portable_binary(
        &self,
        _original_binary: &Path,
        _bios_firmware_path: Option<&Path>,
        _work_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // Par défaut, pas de patching nécessaire
        Ok(None)
    }

    // =========================================
    // Requirements Validation
    // =========================================

    /// Returns the requirements for this emulator.
    fn get_requirements(&self) -> RequirementInfo {
        RequirementInfo::default()
    }

    /// Validates if requirements are met. 
    /// If `source_path` is provided, it attempts to scan and auto-fix (copy files).
    fn validate_requirements(&self, _source_path: Option<&Path>) -> Result<ValidationResult> {
        Ok(ValidationResult {
            valid: true,
            message: "No specific requirements".to_string(),
            fixed: false,
        })
    }
}


pub mod ppsspp;
pub mod pcsx2;
pub mod dolphin;
pub mod duckstation;
pub mod rpcs3;
pub mod ryujinx;
pub mod cemu;
pub mod xemu;
pub mod azahar;
pub mod melonds;
pub mod flycast;
pub mod manager;



