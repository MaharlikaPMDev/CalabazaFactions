pub mod domain;

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};

struct CalabazaFactions;

impl Plugin for CalabazaFactions {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "CalabazaFactions".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["MaharlikaPMDev".into()],
            description:
                "Hardcore factions, diplomacy, economy, claims, and guild warfare for PumpkinMC."
                    .into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("CalabazaFactions loaded; domain and persistence foundation ready");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(CalabazaFactions);
