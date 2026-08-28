use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub factions: FactionConfig,
    pub economy: EconomyConfig,
    pub war: WarConfig,
    pub storage: StorageConfig,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionConfig {
    pub max_members: usize,
    pub starting_power: i64,
    pub power_per_member: i64,
    pub power_loss_on_death: i64,
    pub max_claims_per_power: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub starting_wallet: i64,
    pub war_base_reparation: i64,
    pub pow_base_ransom: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarConfig {
    pub request_hours: u64,
    pub preparation_hours: u64,
    pub ready_countdown_minutes: u64,
    pub battle_minutes: u64,
    pub prisoner_hours: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub max_audit_events: usize,
    pub max_mail_per_faction: usize,
    pub trade_slots: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            factions: FactionConfig {
                max_members: 20,
                starting_power: 10,
                power_per_member: 10,
                power_loss_on_death: 2,
                max_claims_per_power: true,
            },
            economy: EconomyConfig {
                starting_wallet: 1000,
                war_base_reparation: 1000,
                pow_base_ransom: 250,
            },
            war: WarConfig {
                request_hours: 72,
                preparation_hours: 12,
                ready_countdown_minutes: 5,
                battle_minutes: 30,
                prisoner_hours: 24,
            },
            storage: StorageConfig {
                max_audit_events: 5000,
                max_mail_per_faction: 200,
                trade_slots: 54,
            },
        }
    }
}
