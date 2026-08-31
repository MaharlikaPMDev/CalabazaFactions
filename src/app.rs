use crate::{
    config::{Config, EconomyMode},
    domain::*,
    storage,
};
use pumpkin_plugin_api::{Player, Server, inventory::Inventory};
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
    pub second: Option<Location>,
}

#[derive(Clone, Debug)]
pub struct TerritoryView {
    pub origin: Claim,
    pub offset_x: i32,
    pub offset_z: i32,
    pub management: bool,
    pub last_refresh_at: u64,
}

#[derive(Clone, Debug)]
pub struct TerritoryFormView {
    pub view: TerritoryView,
    pub actions: Vec<TerritoryFormAction>,
}

#[derive(Clone, Debug)]
pub enum TerritoryFormAction {
    Pan(i32, i32),
    Inspect(Claim),
    Recenter,
    Refresh,
    Status,
    ToggleManagement,
}

#[derive(Clone, Debug)]
pub struct PendingTerritoryAction {
    pub claim: Claim,
    pub action: String,
    pub expires_at: u64,
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
    pub territory_forms: Mutex<HashMap<u32, TerritoryFormView>>,
    pub menus: Mutex<HashMap<String, String>>,
    pub territory_views: Mutex<HashMap<String, TerritoryView>>,
    pub territory_intents: Mutex<VecDeque<(String, TerritoryFormAction)>>,
    pub territory_reopening: Mutex<HashSet<String>>,
    pub pending_territory: Mutex<HashMap<String, PendingTerritoryAction>>,
    pub trades: Mutex<HashMap<String, TradeView>>,
    pub arena_setup: Mutex<HashMap<String, ArenaSetup>>,
    pub zone_setup: Mutex<HashMap<String, ZoneSetup>>,
    pub last_hits: Mutex<HashMap<i32, (String, u64)>>,
    pub scoreboards: Mutex<HashSet<String>>,
    pub ipc_subscribers: Mutex<HashMap<String, HashSet<String>>>,
    pub delivered_event_sequence: Mutex<u64>,
    pub reconcile_cursor: Mutex<usize>,
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
            territory_forms: Mutex::new(HashMap::new()),
            menus: Mutex::new(HashMap::new()),
            territory_views: Mutex::new(HashMap::new()),
            territory_intents: Mutex::new(VecDeque::new()),
            territory_reopening: Mutex::new(HashSet::new()),
            pending_territory: Mutex::new(HashMap::new()),
            trades: Mutex::new(HashMap::new()),
            arena_setup: Mutex::new(HashMap::new()),
            zone_setup: Mutex::new(HashMap::new()),
            last_hits: Mutex::new(HashMap::new()),
            scoreboards: Mutex::new(HashSet::new()),
            ipc_subscribers: Mutex::new(HashMap::new()),
            delivered_event_sequence: Mutex::new(0),
            reconcile_cursor: Mutex::new(0),
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
    pub fn enqueue_territory_intent(&self, player_id: String, action: TerritoryFormAction) {
        let mut queue = self
            .territory_intents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if queue.len() < 256 && !queue.iter().any(|(queued, _)| queued == &player_id) {
            queue.push_back((player_id, action));
        }
    }
    pub fn remember_player(&self, p: &Player) {
        let id = Self::player_id(p);
        let mut s = self.state.lock().unwrap_or_else(|e| e.into_inner());
        s.player_names.insert(id.clone(), p.get_name());
        if self.config.economy.mode == EconomyMode::Standalone {
            s.wallets
                .entry(id)
                .or_insert(self.config.economy.starting_wallet);
        }
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
        candidate.retain_events(
            Self::now(),
            self.config.ipc.event_retention_count,
            self.config.ipc.event_retention_seconds,
        );
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

    pub fn core_level(&self, faction: &Faction) -> u8 {
        faction.core_level()
    }

    pub fn core_max_lives(&self, faction: &Faction) -> u32 {
        self.config.cores.starting_lives.saturating_add(
            u32::from(self.core_level(faction).saturating_sub(1))
                .saturating_mul(self.config.cores.lives_per_level),
        )
    }

    pub fn core_claim_capacity(&self, faction: &Faction) -> usize {
        self.config.cores.base_claim_capacity.max(9).saturating_add(
            usize::from(self.core_level(faction).saturating_sub(1))
                .saturating_mul(self.config.cores.claims_per_level),
        )
    }

    pub fn initial_core_claims(core: &BlockLocation) -> Vec<Claim> {
        let center = core.claim();
        let mut claims = Vec::with_capacity(9);
        for dz in -1..=1 {
            for dx in -1..=1 {
                claims.push(Claim {
                    world: center.world.clone(),
                    chunk_x: center.chunk_x + dx,
                    chunk_z: center.chunk_z + dz,
                });
            }
        }
        claims
    }

    pub fn core_at(&self, world: &str, x: i32, y: i32, z: i32) -> Option<(String, PhysicalCore)> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let claim = Claim {
            world: world.into(),
            chunk_x: x.div_euclid(16),
            chunk_z: z.div_euclid(16),
        };
        let id = state.core_owners.get(&claim.key())?;
        let core = state.factions.get(id)?.physical_core.as_ref()?;
        (core.location.x == x && core.location.y == y && core.location.z == z)
            .then(|| (id.clone(), core.clone()))
    }

    pub fn core_clearance_owner(&self, world: &str, x: i32, y: i32, z: i32) -> Option<String> {
        let radius = self.config.cores.clearance_outward_blocks.max(0);
        let height = radius;
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let chunk_radius = radius.div_euclid(16) + 1;
        let center_x = x.div_euclid(16);
        let center_z = z.div_euclid(16);
        for dz in -chunk_radius..=chunk_radius {
            for dx in -chunk_radius..=chunk_radius {
                let claim = Claim {
                    world: world.into(),
                    chunk_x: center_x + dx,
                    chunk_z: center_z + dz,
                };
                let Some(id) = state.core_owners.get(&claim.key()) else {
                    continue;
                };
                let Some(core) = state
                    .factions
                    .get(id)
                    .and_then(|faction| faction.physical_core.as_ref())
                else {
                    continue;
                };
                if (x - core.location.x).abs() <= radius
                    && (z - core.location.z).abs() <= radius
                    && y >= core.location.y
                    && y <= core.location.y + height
                {
                    return Some(id.clone());
                }
            }
        }
        None
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
