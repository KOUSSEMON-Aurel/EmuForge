use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::forge::LaunchConfig;

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

    /// Arguments pour activer le mode plein écran.
    /// Implémentation par défaut : --fullscreen (standard pour la plupart des émulateurs).
    fn fullscreen_args(&self) -> Vec<String> {
        vec!["--fullscreen".to_string()]
    }

    /// Prépare l'environnement de l'émulateur (config, BIOS, etc.).
    /// Appelé avant le lancement pour créer les fichiers nécessaires.
    fn setup_environment(&self, _output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        Ok(())
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
}


pub mod ppsspp;
pub mod pcsx2;
pub mod dolphin;
pub mod duckstation;
pub mod rpcs3;
pub mod ryujinx;
pub mod cemu;
pub mod xemu;
pub mod lime3ds;
pub mod melonds;
pub mod redream;
pub mod manager;



