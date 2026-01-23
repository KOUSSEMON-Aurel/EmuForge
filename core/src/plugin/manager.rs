use std::path::{Path};
use crate::plugin::EmulatorPlugin;
// use crate::plugin::ppsspp::PpssppPlugin;
// Future imports
// use crate::plugin::pcsx2::Pcsx2Plugin;
// use crate::plugin::dolphin::DolphinPlugin;
// use crate::plugin::duckstation::DuckStationPlugin;

pub struct PluginManager {
    plugins: Vec<Box<dyn EmulatorPlugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        use crate::plugin::*;
        Self {
            plugins: vec![
                Box::new(ppsspp::PpssppPlugin::new(None)),
                Box::new(pcsx2::Pcsx2Plugin::new(None)),
                Box::new(dolphin::DolphinPlugin::new(None)),
                Box::new(duckstation::DuckStationPlugin::new(None)),
                Box::new(rpcs3::Rpcs3Plugin::new(None)),
                Box::new(ryujinx::RyujinxPlugin::new(None)),
                Box::new(cemu::CemuPlugin::new(None)),
                Box::new(xemu::XemuPlugin::new(None)),
                Box::new(lime3ds::Lime3DSPlugin::new(None)),
                Box::new(melonds::MelonDSPlugin::new(None)),
                Box::new(redream::RedreamPlugin::new(None)),
            ]
        }
    }


    /// Finds a plugin that claims ownership of the provided emulator binary
    /// and returns the reference to the static instance.
    pub fn find_driver_for(&self, binary_path: &Path) -> Option<&dyn EmulatorPlugin> {
        self.plugins.iter().find(|p| p.can_handle(binary_path)).map(|b| b.as_ref())
    }

    /// Finds and configures a NEW instance of the matching plugin setup to use the provided binary.
    /// Utilise clone_with_path pour éviter le match/case répétitif.
    pub fn configured_driver_for(&self, binary_path: &Path) -> Option<Box<dyn EmulatorPlugin>> {
        self.plugins.iter()
            .find(|p| p.can_handle(binary_path))
            .map(|p| p.clone_with_path(binary_path.to_path_buf()))
    }

    /// Finds a plugin by its ID string.
    pub fn get_plugin_by_id(&self, id: &str) -> Option<&dyn EmulatorPlugin> {
        self.plugins.iter().find(|p| p.id() == id).map(|b| b.as_ref())
    }
}
