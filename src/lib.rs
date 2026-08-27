use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};

/// CalabazaFaction is intentionally a small, safe loading skeleton.
/// Gameplay systems are tracked in ROADMAP.md and will be implemented incrementally.
struct CalabazaFaction;

impl Plugin for CalabazaFaction {
    fn new() -> Self { Self }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "CalabazaFaction".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["MaharlikaPMDev".into()],
            description: "Hardcore factions and guild warfare foundation for PumpkinMC.".into(),
            dependencies: vec![],
            permissions: vec![],
        }
    }

    fn on_load(&mut self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        tracing::info!("CalabazaFaction skeleton loaded; gameplay systems are not enabled yet");
        Ok(())
    }
}

pumpkin_plugin_api::register_plugin!(CalabazaFaction);
