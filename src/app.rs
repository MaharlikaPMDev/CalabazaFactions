use crate::{config::Config, domain::*, storage};
use pumpkin_plugin_api::{Player, Server, inventory::Inventory};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub enum ArenaSetup {
    Pair {
        arena: String,
        team1: Option<Location>,
    },
    Spawn {
        arena: String,
        side: u8,
    },
}

#[derive(Clone)]
pub struct ZoneSetup {
    pub id: String,
    pub kind: ZoneKind,
    pub first: Option<Location>,
}

pub enum TradeView {
    Send {
        target: String,
        inventory: Inventory,
    },
    Inbox {
        faction: String,
        inventory: Inventory,
    },
}
pub struct App {
    pub data_dir: PathBuf,
    pub config: Config,
    pub state: Mutex<FactionState>,
    pub forms: Mutex<HashMap<u32, String>>,
    pub menus: Mutex<HashMap<String, String>>,
    pub trades: Mutex<HashMap<String, TradeView>>,
    pub arena_setup: Mutex<HashMap<String, ArenaSetup>>,
    pub zone_setup: Mutex<HashMap<String, ZoneSetup>>,
    pub last_hits: Mutex<HashMap<i32, (String, u64)>>,
    pub scoreboards: Mutex<HashSet<String>>,
}

impl App {
    pub fn load(data_dir: PathBuf) -> Result<Self, String> {
        let (config, state) = storage::load(&data_dir)?;
        storage::save(&data_dir, &state)?;
        Ok(Self {
            data_dir,
            config,
            state: Mutex::new(state),
            forms: Mutex::new(HashMap::new()),
            menus: Mutex::new(HashMap::new()),
            trades: Mutex::new(HashMap::new()),
            arena_setup: Mutex::new(HashMap::new()),
            zone_setup: Mutex::new(HashMap::new()),
            last_hits: Mutex::new(HashMap::new()),
            scoreboards: Mutex::new(HashSet::new()),
        })
    }
    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    pub fn player_id(player: &Player) -> String {
        player.get_id().to_string()
    }
    pub fn remember_player(&self, p: &Player) {
        let id = Self::player_id(p);
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.player_names.insert(id.clone(), p.get_name());
        s.wallets
            .entry(id)
            .or_insert(self.config.economy.starting_wallet);
    }
    pub fn mutate<T>(
        &self,
        actor: &str,
        action: &str,
        f: impl FnOnce(&mut FactionState) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut candidate = state.clone();
        let out = f(&mut candidate)?;
        for faction in candidate.factions.values_mut() {
            let power_bonus = i64::from(faction.upgrade_level(UpgradeKind::Power))
                * self.config.cores.power_per_level;
            faction.max_power = (faction.members.len() as i64
                * self.config.factions.power_per_member
                + power_bonus)
                .max(self.config.factions.starting_power);
            faction.power = faction.power.min(faction.max_power);
        }
        candidate.audit.push(AuditEvent {
            at: Self::now(),
            actor: actor.into(),
            action: action.into(),
            detail: String::new(),
        });
        if candidate.audit.len() > self.config.storage.max_audit_events {
            let excess = candidate.audit.len() - self.config.storage.max_audit_events;
            candidate.audit.drain(0..excess);
        }
        for mailbox in candidate.mail.values_mut() {
            if mailbox.len() > self.config.storage.max_mail_per_faction {
                let excess = mailbox.len() - self.config.storage.max_mail_per_faction;
                mailbox.drain(0..excess);
            }
        }
        storage::save(&self.data_dir, &candidate)?;
        *state = candidate;
        Ok(out)
    }
    pub fn location(p: &Player) -> Location {
        let (x, y, z) = p.get_position();
        Location {
            world: p.get_world().get_id(),
            x,
            y,
            z,
        }
    }
    pub fn claim_at(p: &Player) -> Claim {
        let (x, _, z) = p.get_position();
        Claim {
            world: p.get_world().get_id(),
            chunk_x: (x.floor() as i32).div_euclid(16),
            chunk_z: (z.floor() as i32).div_euclid(16),
        }
    }
    pub fn player_by_entity(server: &Server, id: i32) -> Option<Player> {
        server
            .get_all_players()
            .into_iter()
            .find(|p| p.as_entity().get_id() as i32 == id)
    }
    pub fn can_build_at(&self, p: &Player, x: i32, z: i32) -> bool {
        let id = Self::player_id(p);
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let world = p.get_world().get_id();
        if let Some(zone) = s.zone_at(&world, x, z) {
            return zone.kind == ZoneKind::War;
        }
        let claim = Claim {
            world,
            chunk_x: x.div_euclid(16),
            chunk_z: z.div_euclid(16),
        };
        let Some(owner) = s.claim_owner(&claim) else {
            return true;
        };
        if owner.members.contains_key(&id) {
            return true;
        }
        let Some(visitor) = s.player_faction.get(&id) else {
            return false;
        };
        s.overclaimed(&owner.id) && matches!(s.relation(visitor, &owner.id), Relation::Enemy)
    }

    pub fn can_pvp_at(&self, attacker: &Player, victim: &Player) -> bool {
        let (x, _, z) = victim.get_position();
        let world = victim.get_world().get_id();
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(zone) = s.zone_at(&world, x.floor() as i32, z.floor() as i32) {
            return zone.kind == ZoneKind::War;
        }
        let attacker_id = Self::player_id(attacker);
        let victim_id = Self::player_id(victim);
        match (
            s.player_faction.get(&attacker_id),
            s.player_faction.get(&victim_id),
        ) {
            (Some(a), Some(v)) if a == v => false,
            (Some(a), Some(v)) => !matches!(s.relation(a, v), Relation::Ally | Relation::Truce),
            _ => true,
        }
    }

    pub fn rank_allows(
        &self,
        state: &FactionState,
        player: &str,
        permission: crate::config::RankPermission,
    ) -> bool {
        state
            .role_of(player)
            .is_some_and(|role| self.config.ranks.for_role(role).allows(permission))
    }

    pub fn trade_capacity(&self, faction: &Faction) -> usize {
        self.config.storage.trade_slots
            + usize::from(faction.upgrade_level(UpgradeKind::Vault))
                * self.config.cores.trade_slots_per_level
    }
    pub fn war_notice(&self, id: &str) -> Option<String> {
        let s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let fid = s.player_faction.get(id)?;
        s.wars
            .values()
            .find(|w| {
                matches!(
                    w.status,
                    WarStatus::Requested | WarStatus::Preparing | WarStatus::Active
                ) && (w.attacker == *fid || w.defender == *fid)
            })
            .map(|w| {
                format!(
                    "Faction war {}: {:?}. Use /faction mail and /faction ready.",
                    w.id, w.status
                )
            })
    }
}
