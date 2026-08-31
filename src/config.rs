use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub factions: FactionConfig,
    pub economy: EconomyConfig,
    pub war: WarConfig,
    pub storage: StorageConfig,
    pub protection: ProtectionConfig,
    pub cores: CoreConfig,
    pub territory_ui: TerritoryUiConfig,
    pub scoreboard: ScoreboardConfig,
    pub zones: ZoneConfig,
    pub ipc: IpcConfig,
    pub ranks: RankConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoreboardConfig {
    pub enabled: bool,
}

impl Default for ScoreboardConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FactionConfig {
    pub max_members: usize,
    pub starting_power: i64,
    pub power_per_member: i64,
    pub power_loss_on_death: i64,
    pub max_claims_per_power: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct EconomyConfig {
    pub mode: EconomyMode,
    pub external_plugin: String,
    pub starting_wallet: i64,
    pub war_base_reparation: i64,
    pub pow_base_ransom: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EconomyMode {
    #[default]
    Standalone,
    External,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct WarConfig {
    pub request_hours: u64,
    pub preparation_hours: u64,
    pub ready_countdown_minutes: u64,
    pub battle_minutes: u64,
    pub prisoner_hours: u64,
    pub shield_hours: u64,
    pub cooldown_hours: u64,
    pub post_war_grace_hours: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    pub max_audit_events: usize,
    pub max_mail_per_faction: usize,
    pub trade_slots: usize,
    /// Reserved adapter name. `json` is the portable WASI backend.
    pub backend: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ProtectionConfig {
    pub containers: bool,
    pub pistons: bool,
    pub explosions: bool,
    pub fluids: bool,
    pub entity_grief: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    pub max_upgrade_level: u8,
    pub base_upgrade_cost: i64,
    pub power_per_level: i64,
    pub claims_per_level: usize,
    pub trade_slots_per_level: usize,
    pub shield_hours_per_level: u64,
    pub starting_lives: u32,
    pub lives_per_level: u32,
    pub base_claim_capacity: usize,
    #[serde(default = "default_clearance_outward_blocks")]
    pub clearance_outward_blocks: i32,
    pub hit_cooldown_seconds: u64,
    pub replacement_cooldown_seconds: u64,
    pub replacement_cost: i64,
    pub reconcile_interval_ticks: u64,
    pub reconcile_batch_size: usize,
    pub enemy_core_distance_chunks: i32,
    /// Zero disables the anti-corridor distance cap.
    pub max_claim_distance_from_core: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TerritoryUiConfig {
    pub max_pan_steps: i32,
    pub refresh_cooldown_seconds: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoneConfig {
    pub safe_buffer_chunks: i32,
    pub war_buffer_chunks: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct IpcConfig {
    pub event_retention_count: usize,
    pub event_retention_seconds: u64,
    pub delivery_interval_ticks: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RankPermission {
    Members,
    Territory,
    Economy,
    Diplomacy,
    War,
    Home,
    Trade,
    Core,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RolePermissions {
    pub members: bool,
    pub territory: bool,
    pub economy: bool,
    pub diplomacy: bool,
    pub war: bool,
    pub home: bool,
    pub trade: bool,
    pub core: bool,
}

impl RolePermissions {
    pub fn allows(&self, permission: RankPermission) -> bool {
        match permission {
            RankPermission::Members => self.members,
            RankPermission::Territory => self.territory,
            RankPermission::Economy => self.economy,
            RankPermission::Diplomacy => self.diplomacy,
            RankPermission::War => self.war,
            RankPermission::Home => self.home,
            RankPermission::Trade => self.trade,
            RankPermission::Core => self.core,
        }
    }

    const fn all() -> Self {
        Self {
            members: true,
            territory: true,
            economy: true,
            diplomacy: true,
            war: true,
            home: true,
            trade: true,
            core: true,
        }
    }

    const fn none() -> Self {
        Self {
            members: false,
            territory: false,
            economy: false,
            diplomacy: false,
            war: false,
            home: false,
            trade: false,
            core: false,
        }
    }
}

impl Default for RolePermissions {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RankConfig {
    pub leader: RolePermissions,
    pub officer: RolePermissions,
    pub veteran: RolePermissions,
    pub member: RolePermissions,
    pub recruit: RolePermissions,
}

impl RankConfig {
    pub fn for_role(&self, role: &crate::domain::Role) -> &RolePermissions {
        match role {
            crate::domain::Role::Leader => &self.leader,
            crate::domain::Role::Officer => &self.officer,
            crate::domain::Role::Veteran => &self.veteran,
            crate::domain::Role::Member => &self.member,
            crate::domain::Role::Recruit => &self.recruit,
        }
    }
}

impl Default for FactionConfig {
    fn default() -> Self {
        Self {
            max_members: 20,
            starting_power: 10,
            power_per_member: 10,
            power_loss_on_death: 2,
            max_claims_per_power: true,
        }
    }
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            mode: EconomyMode::Standalone,
            external_plugin: "CalabazaBank".into(),
            starting_wallet: 1000,
            war_base_reparation: 1000,
            pow_base_ransom: 250,
        }
    }
}

impl Default for WarConfig {
    fn default() -> Self {
        Self {
            request_hours: 72,
            preparation_hours: 12,
            ready_countdown_minutes: 5,
            battle_minutes: 30,
            prisoner_hours: 24,
            shield_hours: 8,
            cooldown_hours: 24,
            post_war_grace_hours: 12,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_audit_events: 5000,
            max_mail_per_faction: 200,
            trade_slots: 54,
            backend: "json".into(),
        }
    }
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            containers: true,
            pistons: true,
            explosions: true,
            fluids: true,
            entity_grief: true,
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            max_upgrade_level: 5,
            base_upgrade_cost: 2500,
            power_per_level: 5,
            claims_per_level: 2,
            trade_slots_per_level: 9,
            shield_hours_per_level: 1,
            starting_lives: 10,
            lives_per_level: 5,
            base_claim_capacity: 9,
            clearance_outward_blocks: default_clearance_outward_blocks(),
            hit_cooldown_seconds: 3,
            replacement_cooldown_seconds: 3600,
            replacement_cost: 0,
            reconcile_interval_ticks: 600,
            reconcile_batch_size: 10,
            enemy_core_distance_chunks: 2,
            max_claim_distance_from_core: 0,
        }
    }
}

fn default_clearance_outward_blocks() -> i32 {
    4
}

impl Default for TerritoryUiConfig {
    fn default() -> Self {
        Self {
            max_pan_steps: 5,
            refresh_cooldown_seconds: 2,
        }
    }
}

impl Default for ZoneConfig {
    fn default() -> Self {
        Self {
            safe_buffer_chunks: 1,
            war_buffer_chunks: 0,
        }
    }
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            event_retention_count: 10_000,
            event_retention_seconds: 604_800,
            delivery_interval_ticks: 20,
        }
    }
}

impl Default for RankConfig {
    fn default() -> Self {
        let mut veteran = RolePermissions::none();
        veteran.trade = true;
        let mut member = RolePermissions::none();
        member.trade = true;
        Self {
            leader: RolePermissions::all(),
            officer: RolePermissions::all(),
            veteran,
            member,
            recruit: RolePermissions::none(),
        }
    }
}
