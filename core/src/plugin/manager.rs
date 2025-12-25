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
    /// This fixes the issue where the static plugin doesn't know the user-provided path.
    pub fn configured_driver_for(&self, binary_path: &Path) -> Option<Box<dyn EmulatorPlugin>> {
        let base = self.find_driver_for(binary_path)?;
        let id = base.id();
        
        use crate::plugin::*;
        let path = Some(binary_path.to_path_buf());

        match id {
            "ppsspp" => Some(Box::new(ppsspp::PpssppPlugin::new(path))),
            "pcsx2" => Some(Box::new(pcsx2::Pcsx2Plugin::new(path))),
            "duckstation" => Some(Box::new(duckstation::DuckStationPlugin::new(path))),
            "dolphin" => Some(Box::new(dolphin::DolphinPlugin::new(path))),
            "rpcs3" => Some(Box::new(rpcs3::Rpcs3Plugin::new(path))),
            "ryujinx" => Some(Box::new(ryujinx::RyujinxPlugin::new(path))),
            "cemu" => Some(Box::new(cemu::CemuPlugin::new(path))),
            "xemu" => Some(Box::new(xemu::XemuPlugin::new(path))),
            "lime3ds" => Some(Box::new(lime3ds::Lime3DSPlugin::new(path))),
            "melonds" => Some(Box::new(melonds::MelonDSPlugin::new(path))),
            "redream" => Some(Box::new(redream::RedreamPlugin::new(path))),
            _ => None,
        }
    }
}
