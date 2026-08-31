use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

pub const SCHEMA_VERSION: u32 = 4;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Leader,
    Officer,
    Veteran,
    Member,
    Recruit,
}
impl Role {
    pub fn can_manage(&self) -> bool {
        matches!(self, Self::Leader | Self::Officer)
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "leader" => Some(Self::Leader),
            "officer" => Some(Self::Officer),
            "veteran" => Some(Self::Veteran),
            "member" => Some(Self::Member),
            "recruit" => Some(Self::Recruit),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Neutral,
    Truce,
    Ally,
    Enemy,
}
impl Relation {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "neutral" => Some(Self::Neutral),
            "truce" => Some(Self::Truce),
            "ally" => Some(Self::Ally),
            "enemy" => Some(Self::Enemy),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Claim {
    pub world: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl Claim {
    pub fn key(&self) -> String {
        format!("{}|{}|{}", self.world, self.chunk_x, self.chunk_z)
    }

    pub fn cardinally_adjacent(&self, other: &Self) -> bool {
        self.world == other.world
            && (self.chunk_x - other.chunk_x).abs() + (self.chunk_z - other.chunk_z).abs() == 1
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockLocation {
    pub world: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockLocation {
    pub fn claim(&self) -> Claim {
        Claim {
            world: self.world.clone(),
            chunk_x: self.x.div_euclid(16),
            chunk_z: self.z.div_euclid(16),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreLifecycle {
    #[default]
    AwaitingCore,
    Active,
    Destroyed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhysicalCore {
    pub location: BlockLocation,
    pub lives: u32,
    pub max_lives: u32,
    pub last_hit_at: u64,
    pub established_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClaimSnapshot {
    pub destroyed_at: u64,
    pub claims: Vec<Claim>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FactionEvent {
    pub schema: String,
    pub version: u32,
    pub sequence: u64,
    pub event_type: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    Safe,
    War,
}

impl ZoneKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "safe" | "safe_zone" => Some(Self::Safe),
            "war" | "war_zone" => Some(Self::War),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Zone {
    pub id: String,
    pub kind: ZoneKind,
    pub world: String,
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
    #[serde(default)]
    pub chunk_aligned: bool,
}

impl Zone {
    pub fn contains(&self, world: &str, x: i32, z: i32) -> bool {
        self.world == world
            && (self.min_x..=self.max_x).contains(&x)
            && (self.min_z..=self.max_z).contains(&z)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Arena {
    pub id: String,
    pub team1_spawns: Vec<Location>,
    pub team2_spawns: Vec<Location>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeKind {
    Power,
    Territory,
    Vault,
    Shield,
}

impl UpgradeKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "power" => Some(Self::Power),
            "territory" | "claims" => Some(Self::Territory),
            "vault" | "storage" => Some(Self::Vault),
            "shield" => Some(Self::Shield),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WarPolicyState {
    pub shield_until: u64,
    pub cooldown_until: u64,
    pub grace_until: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub id: String,
    pub name: String,
    pub description: String,
    pub visibility: Visibility,
    pub leader: String,
    pub members: HashMap<String, Role>,
    pub claims: HashSet<Claim>,
    pub power: i64,
    pub max_power: i64,
    pub bank: i64,
    pub prison: Option<Location>,
    pub home: Option<Location>,
    #[serde(default)]
    pub core: Option<Location>,
    #[serde(default)]
    pub physical_core: Option<PhysicalCore>,
    #[serde(default)]
    pub core_lifecycle: CoreLifecycle,
    #[serde(default)]
    pub core_destroyed_at: u64,
    #[serde(default)]
    pub last_claim_snapshot: Option<ClaimSnapshot>,
    #[serde(default)]
    pub banner: Option<TradeItem>,
    #[serde(default)]
    pub upgrades: HashMap<UpgradeKind, u8>,
    #[serde(default)]
    pub war_policy: WarPolicyState,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invitation {
    pub faction: String,
    pub player: String,
    pub expires_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Application {
    pub faction: String,
    pub player: String,
    pub created_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mail {
    pub id: u64,
    pub subject: String,
    pub body: String,
    pub created_at: u64,
    pub read: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeItem {
    pub registry_key: String,
    pub count: u8,
    #[serde(default)]
    pub components: Vec<TradeComponent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TradeComponent {
    /// Stable index from Pumpkin's pinned `data-component` WIT enum.
    pub id: u16,
    pub value: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prisoner {
    pub player: String,
    pub captor_faction: String,
    pub ransom: i64,
    pub release_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WarStatus {
    Requested,
    Preparing,
    Active,
    Finished,
    Declined,
    Expired,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct War {
    pub id: String,
    pub attacker: String,
    pub defender: String,
    pub forced: bool,
    pub status: WarStatus,
    pub requested_at: u64,
    pub request_expires_at: u64,
    pub preparation_ends_at: Option<u64>,
    pub battle_ends_at: Option<u64>,
    pub ready: HashSet<String>,
    pub prisoners: HashMap<String, Prisoner>,
    pub winner: Option<String>,
    pub loser: Option<String>,
    pub reparations: i64,
    #[serde(default)]
    pub arena_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub at: u64,
    pub actor: String,
    pub action: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionState {
    pub schema_version: u32,
    pub next_mail_id: u64,
    pub factions: HashMap<String, Faction>,
    pub player_faction: HashMap<String, String>,
    pub player_names: HashMap<String, String>,
    pub wallets: HashMap<String, i64>,
    pub invitations: Vec<Invitation>,
    pub applications: Vec<Application>,
    pub relations: HashMap<String, Relation>,
    pub wars: HashMap<String, War>,
    pub mail: HashMap<String, Vec<Mail>>,
    pub trade: HashMap<String, Vec<TradeItem>>,
    pub arena: Option<Location>,
    #[serde(default)]
    pub arena_team2: Option<Location>,
    #[serde(default)]
    pub arenas: HashMap<String, Arena>,
    #[serde(default)]
    pub arena_cursor: usize,
    #[serde(default)]
    pub zones: HashMap<String, Zone>,
    #[serde(default)]
    pub claim_owners: HashMap<String, String>,
    #[serde(default)]
    pub core_owners: HashMap<String, String>,
    #[serde(default = "default_next_event_sequence")]
    pub next_event_sequence: u64,
    #[serde(default)]
    pub events: Vec<FactionEvent>,
    pub audit: Vec<AuditEvent>,
}

fn default_next_event_sequence() -> u64 {
    1
}
impl Default for FactionState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            next_mail_id: 1,
            factions: HashMap::new(),
            player_faction: HashMap::new(),
            player_names: HashMap::new(),
            wallets: HashMap::new(),
            invitations: vec![],
            applications: vec![],
            relations: HashMap::new(),
            wars: HashMap::new(),
            mail: HashMap::new(),
            trade: HashMap::new(),
            arena: None,
            arena_team2: None,
            arenas: HashMap::new(),
            arena_cursor: 0,
            zones: HashMap::new(),
            claim_owners: HashMap::new(),
            core_owners: HashMap::new(),
            next_event_sequence: 1,
            events: vec![],
            audit: vec![],
        }
    }
}

impl Faction {
    pub fn upgrade_level(&self, kind: UpgradeKind) -> u8 {
        self.upgrades.get(&kind).copied().unwrap_or(0)
    }

    pub fn has_active_core(&self) -> bool {
        self.core_lifecycle == CoreLifecycle::Active && self.physical_core.is_some()
    }

    pub fn core_level(&self) -> u8 {
        1u8.saturating_add(self.upgrade_level(UpgradeKind::Territory))
    }
}

impl FactionState {
    pub fn migrate(&mut self) {
        let previous_schema = self.schema_version;
        if self.arenas.is_empty()
            && let (Some(team1), Some(team2)) = (self.arena.take(), self.arena_team2.take())
        {
            self.arenas.insert(
                "default".into(),
                Arena {
                    id: "default".into(),
                    team1_spawns: vec![team1],
                    team2_spawns: vec![team2],
                    enabled: true,
                },
            );
        }
        if previous_schema < 4 {
            for faction in self.factions.values_mut() {
                if !faction.claims.is_empty() {
                    faction.last_claim_snapshot = Some(ClaimSnapshot {
                        destroyed_at: 0,
                        claims: faction.claims.iter().cloned().collect(),
                    });
                    faction.claims.clear();
                }
                faction.physical_core = None;
                faction.core_lifecycle = CoreLifecycle::AwaitingCore;
                faction.core_destroyed_at = 0;
            }
        }
        self.rebuild_claim_index();
        self.rebuild_core_index();
        if self.next_event_sequence == 0 {
            self.next_event_sequence = 1;
        }
        self.schema_version = SCHEMA_VERSION;
    }

    pub fn normalize(value: &str) -> String {
        value
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect()
    }
    fn relation_key(a: &str, b: &str) -> String {
        if a <= b {
            format!("{a}|{b}")
        } else {
            format!("{b}|{a}")
        }
    }
    pub fn relation(&self, a: &str, b: &str) -> Relation {
        self.relations
            .get(&Self::relation_key(a, b))
            .cloned()
            .unwrap_or(Relation::Neutral)
    }
    pub fn set_relation(&mut self, a: &str, b: &str, relation: Relation) -> Result<(), String> {
        if a == b {
            return Err("a faction cannot relate to itself".into());
        }
        if !self.factions.contains_key(a) || !self.factions.contains_key(b) {
            return Err("faction not found".into());
        }
        self.relations.insert(Self::relation_key(a, b), relation);
        Ok(())
    }
    pub fn faction_of(&self, player: &str) -> Option<&Faction> {
        self.player_faction
            .get(player)
            .and_then(|id| self.factions.get(id))
    }
    pub fn role_of(&self, player: &str) -> Option<&Role> {
        self.faction_of(player).and_then(|f| f.members.get(player))
    }
    pub fn create(
        &mut self,
        name: &str,
        leader: &str,
        visibility: Visibility,
        now: u64,
        starting_power: i64,
    ) -> Result<String, String> {
        let id = Self::normalize(name);
        if id.len() < 3 || id.len() > 24 {
            return Err("faction name must be 3-24 letters, numbers, _ or -".into());
        }
        if self.factions.contains_key(&id) {
            return Err("faction already exists".into());
        }
        if self.player_faction.contains_key(leader) {
            return Err("you already belong to a faction".into());
        }
        let mut members = HashMap::new();
        members.insert(leader.into(), Role::Leader);
        self.factions.insert(
            id.clone(),
            Faction {
                id: id.clone(),
                name: name.trim().into(),
                description: String::new(),
                visibility,
                leader: leader.into(),
                members,
                claims: HashSet::new(),
                power: starting_power,
                max_power: starting_power,
                bank: 0,
                prison: None,
                home: None,
                core: None,
                physical_core: None,
                core_lifecycle: CoreLifecycle::AwaitingCore,
                core_destroyed_at: 0,
                last_claim_snapshot: None,
                banner: None,
                upgrades: HashMap::new(),
                war_policy: WarPolicyState::default(),
                created_at: now,
            },
        );
        self.player_faction.insert(leader.into(), id.clone());
        Ok(id)
    }
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let faction = self.factions.remove(id).ok_or("faction not found")?;
        for claim in &faction.claims {
            self.claim_owners.remove(&claim.key());
        }
        self.core_owners.retain(|_, owner| owner != id);
        for p in faction.members.keys() {
            self.player_faction.remove(p);
        }
        self.relations.retain(|k, _| !k.split('|').any(|v| v == id));
        self.applications.retain(|a| a.faction != id);
        self.invitations.retain(|i| i.faction != id);
        Ok(())
    }
    pub fn invite(&mut self, faction: &str, player: &str, expires_at: u64) -> Result<(), String> {
        let faction_state = self.factions.get(faction).ok_or("faction not found")?;
        if !faction_state.has_active_core() {
            return Err("the faction must establish an active core before recruiting".into());
        }
        if self.player_faction.contains_key(player) {
            return Err("player already belongs to a faction".into());
        }
        self.invitations
            .retain(|i| !(i.faction == faction && i.player == player));
        self.invitations.push(Invitation {
            faction: faction.into(),
            player: player.into(),
            expires_at,
        });
        Ok(())
    }
    pub fn apply(&mut self, faction: &str, player: &str, now: u64) -> Result<(), String> {
        let f = self.factions.get(faction).ok_or("faction not found")?;
        if !f.has_active_core() {
            return Err("that faction cannot accept applications until its core is active".into());
        }
        if f.visibility == Visibility::Private {
            return Err("that faction is invitation-only".into());
        }
        if self.player_faction.contains_key(player) {
            return Err("you already belong to a faction".into());
        }
        self.applications
            .retain(|a| !(a.faction == faction && a.player == player));
        self.applications.push(Application {
            faction: faction.into(),
            player: player.into(),
            created_at: now,
        });
        Ok(())
    }
    pub fn join(
        &mut self,
        faction: &str,
        player: &str,
        now: u64,
        max_members: usize,
        require_invite: bool,
    ) -> Result<(), String> {
        if self.player_faction.contains_key(player) {
            return Err("player already belongs to a faction".into());
        }
        let f = self.factions.get(faction).ok_or("faction not found")?;
        if !f.has_active_core() {
            return Err("that faction cannot receive members until its core is active".into());
        }
        if f.members.len() >= max_members {
            return Err("faction is full".into());
        }
        if require_invite
            && !self
                .invitations
                .iter()
                .any(|i| i.faction == faction && i.player == player && i.expires_at >= now)
        {
            return Err("no valid invitation found".into());
        }
        self.factions
            .get_mut(faction)
            .unwrap()
            .members
            .insert(player.into(), Role::Recruit);
        self.player_faction.insert(player.into(), faction.into());
        self.invitations
            .retain(|i| !(i.faction == faction && i.player == player));
        self.applications
            .retain(|a| !(a.faction == faction && a.player == player));
        Ok(())
    }
    pub fn leave(&mut self, player: &str) -> Result<(), String> {
        let id = self
            .player_faction
            .get(player)
            .cloned()
            .ok_or("you are not in a faction")?;
        if self.factions.get(&id).is_some_and(|f| f.leader == player) {
            return Err("leader must transfer leadership or disband".into());
        }
        self.factions.get_mut(&id).unwrap().members.remove(player);
        self.player_faction.remove(player);
        Ok(())
    }
    pub fn claim(&mut self, id: &str, claim: Claim) -> Result<(), String> {
        self.claim_with_bonus(id, claim, 0)
    }

    pub fn claim_with_bonus(
        &mut self,
        id: &str,
        claim: Claim,
        bonus_claims: usize,
    ) -> Result<(), String> {
        if self.claim_owners.contains_key(&claim.key()) {
            return Err("chunk is already claimed".into());
        }
        let f = self.factions.get_mut(id).ok_or("faction not found")?;
        if !f.has_active_core() {
            return Err("establish an active faction core before claiming territory".into());
        }
        if f.claims.len() >= f.power.max(0) as usize + bonus_claims {
            return Err("not enough faction power".into());
        }
        if !f
            .claims
            .iter()
            .any(|owned| owned.cardinally_adjacent(&claim))
        {
            return Err("new territory must share a north, south, east, or west edge".into());
        }
        self.claim_owners.insert(claim.key(), id.into());
        f.claims.insert(claim);
        Ok(())
    }

    pub fn strategic_claim(
        &mut self,
        id: &str,
        claim: Claim,
        capacity: usize,
    ) -> Result<(), String> {
        if self.claim_owners.contains_key(&claim.key()) {
            return Err("chunk is already claimed".into());
        }
        let faction = self.factions.get(id).ok_or("faction not found")?;
        if !faction.has_active_core() {
            return Err("establish an active faction core before claiming territory".into());
        }
        if faction.claims.len() >= capacity {
            return Err("your core level cannot support another chunk".into());
        }
        if !faction
            .claims
            .iter()
            .any(|owned| owned.cardinally_adjacent(&claim))
        {
            return Err("new territory must share a north, south, east, or west edge".into());
        }
        self.claim_owners.insert(claim.key(), id.into());
        self.factions.get_mut(id).unwrap().claims.insert(claim);
        Ok(())
    }

    pub fn strategic_overclaim(
        &mut self,
        attacker: &str,
        claim: Claim,
        capacity: usize,
    ) -> Result<String, String> {
        let owner = self
            .claim_owner(&claim)
            .map(|faction| faction.id.clone())
            .ok_or("chunk is wilderness")?;
        if owner == attacker {
            return Err("your faction already owns this chunk".into());
        }
        if self.relation(attacker, &owner) != Relation::Enemy {
            return Err("only enemy territory can be overclaimed".into());
        }
        if !self.overclaimed(&owner) {
            return Err("target faction is not overclaimed".into());
        }
        let faction = self.factions.get(attacker).ok_or("faction not found")?;
        if !faction.has_active_core() {
            return Err("establish an active faction core before overclaiming".into());
        }
        if faction.claims.len() >= capacity {
            return Err("your core level cannot support another chunk".into());
        }
        if !faction
            .claims
            .iter()
            .any(|owned| owned.cardinally_adjacent(&claim))
        {
            return Err("overclaimed territory must touch your existing territory".into());
        }
        if !self.removal_keeps_connected(&owner, &claim) {
            return Err("overclaim would disconnect the target faction from its core".into());
        }
        self.factions.get_mut(&owner).unwrap().claims.remove(&claim);
        self.factions
            .get_mut(attacker)
            .unwrap()
            .claims
            .insert(claim.clone());
        self.claim_owners.insert(claim.key(), attacker.into());
        Ok(owner)
    }
    pub fn overclaim(&mut self, attacker: &str, claim: Claim) -> Result<String, String> {
        self.overclaim_with_bonus(attacker, claim, 0)
    }

    pub fn overclaim_with_bonus(
        &mut self,
        attacker: &str,
        claim: Claim,
        bonus_claims: usize,
    ) -> Result<String, String> {
        let owner = self
            .claim_owner(&claim)
            .map(|f| f.id.clone())
            .ok_or("chunk is wilderness")?;
        if owner == attacker {
            return Err("your faction already owns this chunk".into());
        }
        if self.relation(attacker, &owner) != Relation::Enemy {
            return Err("only enemy territory can be overclaimed".into());
        }
        if !self.overclaimed(&owner) {
            return Err("target faction is not overclaimed".into());
        }
        let attacking = self.factions.get(attacker).ok_or("faction not found")?;
        if !attacking.has_active_core() {
            return Err("establish an active faction core before overclaiming".into());
        }
        if attacking.claims.len() >= attacking.power.max(0) as usize + bonus_claims {
            return Err("your faction lacks power for another claim".into());
        }
        if !attacking
            .claims
            .iter()
            .any(|owned| owned.cardinally_adjacent(&claim))
        {
            return Err("overclaimed territory must touch your existing territory".into());
        }
        if !self.removal_keeps_connected(&owner, &claim) {
            return Err("overclaim would disconnect the target faction from its core".into());
        }
        self.factions.get_mut(&owner).unwrap().claims.remove(&claim);
        self.factions
            .get_mut(attacker)
            .unwrap()
            .claims
            .insert(claim.clone());
        self.claim_owners.insert(claim.key(), attacker.into());
        Ok(owner)
    }
    pub fn claim_owner(&self, claim: &Claim) -> Option<&Faction> {
        self.claim_owners
            .get(&claim.key())
            .and_then(|id| self.factions.get(id))
            .filter(|faction| faction.has_active_core())
    }
    pub fn overclaimed(&self, id: &str) -> bool {
        self.factions
            .get(id)
            .is_some_and(|f| f.claims.len() > f.power.max(0) as usize)
    }

    pub fn rebuild_claim_index(&mut self) {
        self.claim_owners.clear();
        let mut faction_ids = self.factions.keys().cloned().collect::<Vec<_>>();
        faction_ids.sort();
        for id in faction_ids {
            if let Some(faction) = self.factions.get_mut(&id) {
                let mut conflicts = Vec::new();
                for claim in &faction.claims {
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        self.claim_owners.entry(claim.key())
                    {
                        entry.insert(id.clone());
                    } else {
                        conflicts.push(claim.clone());
                    }
                }
                for claim in conflicts {
                    faction.claims.remove(&claim);
                }
            }
        }
    }

    pub fn rebuild_core_index(&mut self) {
        self.core_owners.clear();
        for (id, faction) in &self.factions {
            if let Some(core) = faction
                .physical_core
                .as_ref()
                .filter(|_| faction.has_active_core())
            {
                self.core_owners
                    .insert(core.location.claim().key(), id.clone());
            }
        }
    }

    pub fn establish_core(
        &mut self,
        id: &str,
        core: PhysicalCore,
        initial_claims: Vec<Claim>,
        now: u64,
    ) -> Result<(), String> {
        let faction = self.factions.get(id).ok_or("faction not found")?;
        if faction.has_active_core() {
            return Err("the faction already has an active core".into());
        }
        if faction.core_destroyed_at > 0 && faction.core_destroyed_at > now {
            return Err("the faction core replacement cooldown is still active".into());
        }
        for claim in &initial_claims {
            if let Some(owner) = self.claim_owners.get(&claim.key())
                && owner != id
            {
                return Err(format!("initial core territory conflicts with {owner}"));
            }
        }
        let faction = self.factions.get_mut(id).ok_or("faction not found")?;
        faction.physical_core = Some(core.clone());
        faction.core_lifecycle = CoreLifecycle::Active;
        faction.core_destroyed_at = 0;
        faction.core = Some(Location {
            world: core.location.world.clone(),
            x: f64::from(core.location.x) + 0.5,
            y: f64::from(core.location.y),
            z: f64::from(core.location.z) + 0.5,
        });
        for claim in initial_claims {
            self.claim_owners.insert(claim.key(), id.into());
            faction.claims.insert(claim);
        }
        self.core_owners
            .insert(core.location.claim().key(), id.into());
        Ok(())
    }

    pub fn unclaim_connected(&mut self, id: &str, claim: &Claim) -> Result<(), String> {
        let faction = self.factions.get(id).ok_or("faction not found")?;
        if faction
            .physical_core
            .as_ref()
            .is_some_and(|core| core.location.claim() == *claim)
        {
            return Err("the core chunk cannot be released".into());
        }
        if !faction.claims.contains(claim) {
            return Err("your faction does not own this chunk".into());
        }
        if !self.removal_keeps_connected(id, claim) {
            return Err("releasing this chunk would disconnect territory from the core".into());
        }
        self.factions.get_mut(id).unwrap().claims.remove(claim);
        self.claim_owners.remove(&claim.key());
        Ok(())
    }

    pub fn removal_keeps_connected(&self, id: &str, removed: &Claim) -> bool {
        let Some(faction) = self.factions.get(id) else {
            return false;
        };
        let Some(core_claim) = faction
            .physical_core
            .as_ref()
            .map(|core| core.location.claim())
        else {
            return false;
        };
        if &core_claim == removed {
            return false;
        }
        let remaining = faction
            .claims
            .iter()
            .filter(|claim| *claim != removed)
            .cloned()
            .collect::<HashSet<_>>();
        if remaining.is_empty() || !remaining.contains(&core_claim) {
            return false;
        }
        let mut seen = HashSet::from([core_claim.clone()]);
        let mut queue = VecDeque::from([core_claim]);
        while let Some(current) = queue.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let next = Claim {
                    world: current.world.clone(),
                    chunk_x: current.chunk_x + dx,
                    chunk_z: current.chunk_z + dz,
                };
                if remaining.contains(&next) && seen.insert(next.clone()) {
                    queue.push_back(next);
                }
            }
        }
        seen.len() == remaining.len()
    }

    pub fn destroy_core(
        &mut self,
        id: &str,
        now: u64,
        replacement_available_at: u64,
    ) -> Result<Vec<Claim>, String> {
        let faction = self.factions.get_mut(id).ok_or("faction not found")?;
        if !faction.has_active_core() {
            return Err("the faction core is not active".into());
        }
        let claims = faction.claims.iter().cloned().collect::<Vec<_>>();
        for claim in &claims {
            self.claim_owners.remove(&claim.key());
        }
        faction.last_claim_snapshot = Some(ClaimSnapshot {
            destroyed_at: now,
            claims: claims.clone(),
        });
        faction.claims.clear();
        faction.physical_core = None;
        faction.core = None;
        faction.core_lifecycle = CoreLifecycle::Destroyed;
        faction.core_destroyed_at = replacement_available_at;
        self.core_owners.retain(|_, owner| owner != id);
        Ok(claims)
    }

    pub fn push_event(&mut self, event_type: &str, timestamp: u64, data: serde_json::Value) -> u64 {
        let sequence = self.next_event_sequence.max(1);
        self.next_event_sequence = sequence.saturating_add(1);
        self.events.push(FactionEvent {
            schema: "calabazafactions.event".into(),
            version: 1,
            sequence,
            event_type: event_type.into(),
            timestamp,
            data,
        });
        sequence
    }

    pub fn retain_events(&mut self, now: u64, max_count: usize, max_age: u64) {
        let oldest = now.saturating_sub(max_age);
        self.events.retain(|event| event.timestamp >= oldest);
        if self.events.len() > max_count {
            let excess = self.events.len() - max_count;
            self.events.drain(0..excess);
        }
    }
    pub fn send_mail(&mut self, target: &str, subject: &str, body: &str, now: u64) {
        let id = self.next_mail_id;
        self.next_mail_id += 1;
        self.mail.entry(target.into()).or_default().push(Mail {
            id,
            subject: subject.into(),
            body: body.into(),
            created_at: now,
            read: false,
        });
    }
    pub fn active_war_between(&self, a: &str, b: &str) -> Option<&War> {
        self.wars.values().find(|w| {
            w.status == WarStatus::Active
                && ((w.attacker == a && w.defender == b) || (w.attacker == b && w.defender == a))
        })
    }
    pub fn war_slot_busy(&self) -> bool {
        self.wars.values().any(|war| {
            matches!(
                war.status,
                WarStatus::Requested | WarStatus::Preparing | WarStatus::Active
            )
        })
    }

    pub fn zone_at(&self, world: &str, x: i32, z: i32) -> Option<&Zone> {
        self.zones
            .values()
            .filter(|zone| zone.contains(world, x, z))
            .max_by_key(|zone| matches!(zone.kind, ZoneKind::Safe))
    }

    pub fn environmental_protected(&self, world: Option<&str>, x: i32, z: i32) -> bool {
        if let Some(world) = world {
            if let Some(zone) = self.zone_at(world, x, z) {
                return zone.kind == ZoneKind::Safe;
            }
            let claim = Claim {
                world: world.into(),
                chunk_x: x.div_euclid(16),
                chunk_z: z.div_euclid(16),
            };
            return self.claim_owner(&claim).is_some();
        }

        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);
        self.zones
            .values()
            .any(|zone| zone.kind == ZoneKind::Safe && zone.contains(&zone.world, x, z))
            || self.factions.values().any(|faction| {
                faction
                    .claims
                    .iter()
                    .any(|claim| claim.chunk_x == chunk_x && claim.chunk_z == chunk_z)
            })
    }

    pub fn usable_arena_ids(&self) -> Vec<String> {
        let mut ids = self
            .arenas
            .iter()
            .filter(|(_, arena)| {
                arena.enabled && !arena.team1_spawns.is_empty() && !arena.team2_spawns.is_empty()
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        ids.sort();
        ids
    }

    pub fn select_arena(&mut self) -> Result<String, String> {
        let ids = self.usable_arena_ids();
        if ids.is_empty() {
            return Err("an admin must configure an arena with spawn groups first".into());
        }
        let id = ids[self.arena_cursor % ids.len()].clone();
        self.arena_cursor = self.arena_cursor.wrapping_add(1);
        Ok(id)
    }

    pub fn war_block_reason(&self, attacker: &str, defender: &str, now: u64) -> Option<String> {
        let attacker = self.factions.get(attacker)?;
        let defender = self.factions.get(defender)?;
        if attacker.war_policy.shield_until > now {
            return Some("your faction must lower or wait out its active war shield".into());
        }
        if attacker.war_policy.cooldown_until > now {
            return Some("your faction is still on war cooldown".into());
        }
        if attacker.war_policy.grace_until > now {
            return Some("your faction is still in its post-war grace period".into());
        }
        if defender.war_policy.shield_until > now {
            return Some("the target faction has an active war shield".into());
        }
        if defender.war_policy.cooldown_until > now {
            return Some("the target faction is still on war cooldown".into());
        }
        if defender.war_policy.grace_until > now {
            return Some("the target faction is still in its post-war grace period".into());
        }
        None
    }

    pub fn set_zone(
        &mut self,
        id: &str,
        kind: ZoneKind,
        first: &Location,
        second: &Location,
    ) -> Result<(), String> {
        if first.world != second.world {
            return Err("both zone corners must be in the same world".into());
        }
        let id = Self::normalize(id);
        if id.is_empty() {
            return Err("zone name must contain a letter or number".into());
        }
        self.zones.insert(
            id.clone(),
            Zone {
                id,
                kind,
                world: first.world.clone(),
                min_x: first.x.floor().min(second.x.floor()) as i32,
                min_z: first.z.floor().min(second.z.floor()) as i32,
                max_x: first.x.floor().max(second.x.floor()) as i32,
                max_z: first.z.floor().max(second.z.floor()) as i32,
                chunk_aligned: false,
            },
        );
        Ok(())
    }

    pub fn set_chunk_zone(
        &mut self,
        id: &str,
        kind: ZoneKind,
        first: &Location,
        second: &Location,
        buffer_chunks: i32,
    ) -> Result<usize, String> {
        if first.world != second.world {
            return Err("both zone corners must be in the same world".into());
        }
        let id = Self::normalize(id);
        if id.is_empty() {
            return Err("zone name must contain a letter or number".into());
        }
        let buffer = buffer_chunks.max(0);
        let first_x = (first.x.floor() as i32).div_euclid(16);
        let first_z = (first.z.floor() as i32).div_euclid(16);
        let second_x = (second.x.floor() as i32).div_euclid(16);
        let second_z = (second.z.floor() as i32).div_euclid(16);
        let min_chunk_x = first_x.min(second_x) - buffer;
        let max_chunk_x = first_x.max(second_x) + buffer;
        let min_chunk_z = first_z.min(second_z) - buffer;
        let max_chunk_z = first_z.max(second_z) + buffer;
        let count = usize::try_from(max_chunk_x - min_chunk_x + 1).unwrap_or(usize::MAX)
            * usize::try_from(max_chunk_z - min_chunk_z + 1).unwrap_or(usize::MAX);
        self.zones.insert(
            id.clone(),
            Zone {
                id,
                kind,
                world: first.world.clone(),
                min_x: min_chunk_x * 16,
                min_z: min_chunk_z * 16,
                max_x: max_chunk_x * 16 + 15,
                max_z: max_chunk_z * 16 + 15,
                chunk_aligned: true,
            },
        );
        Ok(count)
    }
}

pub trait FactionLookup {
    fn faction_id(&self, player: &str) -> Option<&str>;
    fn faction_name(&self, player: &str) -> Option<&str>;
    fn relation_between_players(&self, a: &str, b: &str) -> Relation;
}
impl FactionLookup for FactionState {
    fn faction_id(&self, p: &str) -> Option<&str> {
        self.player_faction.get(p).map(String::as_str)
    }
    fn faction_name(&self, p: &str) -> Option<&str> {
        self.faction_of(p).map(|f| f.name.as_str())
    }
    fn relation_between_players(&self, a: &str, b: &str) -> Relation {
        match (self.faction_id(a), self.faction_id(b)) {
            (Some(x), Some(y)) if x == y => Relation::Ally,
            (Some(x), Some(y)) => self.relation(x, y),
            _ => Relation::Neutral,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn establish(state: &mut FactionState, id: &str, chunk_x: i32) {
        let location = BlockLocation {
            world: "world".into(),
            x: chunk_x * 16 + 8,
            y: 64,
            z: 8,
        };
        let center = location.claim();
        let claims = (-1..=1)
            .flat_map(|dz| {
                let world = center.world.clone();
                (-1..=1).map(move |dx| Claim {
                    world: world.clone(),
                    chunk_x: center.chunk_x + dx,
                    chunk_z: center.chunk_z + dz,
                })
            })
            .collect();
        state
            .establish_core(
                id,
                PhysicalCore {
                    location,
                    lives: 10,
                    max_lives: 10,
                    last_hit_at: 0,
                    established_at: 1,
                },
                claims,
                1,
            )
            .unwrap();
    }

    #[test]
    fn lifecycle() {
        let mut s = FactionState::default();
        let id = s
            .create("Knights", "alice", Visibility::Private, 1, 10)
            .unwrap();
        establish(&mut s, &id, 0);
        s.invite(&id, "bob", 100).unwrap();
        s.join(&id, "bob", 2, 20, true).unwrap();
        assert_eq!(s.faction_name("bob"), Some("Knights"));
        s.leave("bob").unwrap();
    }
    #[test]
    fn private_rejects_application() {
        let mut s = FactionState::default();
        s.create("Knights", "alice", Visibility::Private, 1, 10)
            .unwrap();
        assert!(s.apply("knights", "bob", 2).is_err());
    }
    #[test]
    fn claim_power() {
        let mut s = FactionState::default();
        let id = s
            .create("Knights", "alice", Visibility::Public, 1, 10)
            .unwrap();
        establish(&mut s, &id, 0);
        s.strategic_claim(
            "knights",
            Claim {
                world: "world".into(),
                chunk_x: 2,
                chunk_z: 0,
            },
            10,
        )
        .unwrap();
        assert!(
            s.strategic_claim(
                "knights",
                Claim {
                    world: "world".into(),
                    chunk_x: 3,
                    chunk_z: 0
                },
                10,
            )
            .is_err()
        );
    }
    #[test]
    fn relation_is_symmetric() {
        let mut s = FactionState::default();
        s.create("Aaa", "a", Visibility::Public, 1, 10).unwrap();
        s.create("Bbb", "b", Visibility::Public, 1, 10).unwrap();
        s.set_relation("aaa", "bbb", Relation::Enemy).unwrap();
        assert_eq!(s.relation("bbb", "aaa"), Relation::Enemy);
    }
    #[test]
    fn enemy_can_take_overclaimed_land() {
        let mut s = FactionState::default();
        s.create("Aaa", "a", Visibility::Public, 1, 10).unwrap();
        s.create("Bbb", "b", Visibility::Public, 1, 10).unwrap();
        establish(&mut s, "aaa", 0);
        establish(&mut s, "bbb", 4);
        let claim = Claim {
            world: "world".into(),
            chunk_x: 2,
            chunk_z: 0,
        };
        s.strategic_claim("aaa", claim.clone(), 10).unwrap();
        s.factions.get_mut("aaa").unwrap().power = 0;
        s.set_relation("aaa", "bbb", Relation::Enemy).unwrap();
        assert_eq!(s.strategic_overclaim("bbb", claim, 10).unwrap(), "aaa");
    }

    #[test]
    fn awaiting_core_blocks_every_recruitment_path() {
        let mut state = FactionState::default();
        let id = state
            .create("Knights", "alice", Visibility::Public, 1, 10)
            .unwrap();
        assert!(state.invite(&id, "bob", 100).is_err());
        assert!(state.apply(&id, "bob", 2).is_err());
        assert!(state.join(&id, "bob", 2, 20, false).is_err());
        establish(&mut state, &id, 0);
        assert!(state.invite(&id, "bob", 100).is_ok());
    }

    #[test]
    fn strategic_claims_require_cardinal_contact_and_stay_connected() {
        let mut state = FactionState::default();
        let id = state
            .create("Knights", "alice", Visibility::Public, 1, 10)
            .unwrap();
        establish(&mut state, &id, 0);
        let diagonal = Claim {
            world: "world".into(),
            chunk_x: 2,
            chunk_z: 2,
        };
        assert!(state.strategic_claim(&id, diagonal, 12).is_err());
        for chunk_x in [2, 3] {
            state
                .strategic_claim(
                    &id,
                    Claim {
                        world: "world".into(),
                        chunk_x,
                        chunk_z: 0,
                    },
                    12,
                )
                .unwrap();
        }
        assert!(
            state
                .unclaim_connected(
                    &id,
                    &Claim {
                        world: "world".into(),
                        chunk_x: 2,
                        chunk_z: 0,
                    },
                )
                .is_err()
        );
    }

    #[test]
    fn destroyed_core_preserves_faction_and_snapshots_claims() {
        let mut state = FactionState::default();
        let id = state
            .create("Knights", "alice", Visibility::Public, 1, 10)
            .unwrap();
        establish(&mut state, &id, 0);
        let removed = state.destroy_core(&id, 50, 100).unwrap();
        let faction = &state.factions[&id];
        assert_eq!(removed.len(), 9);
        assert_eq!(faction.members.len(), 1);
        assert_eq!(faction.core_lifecycle, CoreLifecycle::Destroyed);
        assert!(faction.claims.is_empty());
        assert_eq!(
            faction.last_claim_snapshot.as_ref().unwrap().claims.len(),
            9
        );
        assert!(state.claim_owners.is_empty());
    }

    #[test]
    fn event_journal_is_monotonic_and_bounded() {
        let mut state = FactionState::default();
        assert_eq!(
            state.push_event("faction.created", 10, serde_json::json!({})),
            1
        );
        assert_eq!(
            state.push_event("core.established", 20, serde_json::json!({})),
            2
        );
        assert_eq!(
            state.push_event("territory.claimed", 30, serde_json::json!({})),
            3
        );
        state.retain_events(30, 2, 100);
        assert_eq!(
            state
                .events
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            [2, 3]
        );
    }

    #[test]
    fn chunk_zone_expands_selected_bounds_and_buffer_explicitly() {
        let mut state = FactionState::default();
        let first = Location {
            world: "world".into(),
            x: 17.0,
            y: 64.0,
            z: 17.0,
        };
        let second = Location {
            world: "world".into(),
            x: 31.0,
            y: 64.0,
            z: 31.0,
        };
        assert_eq!(
            state
                .set_chunk_zone("spawn", ZoneKind::Safe, &first, &second, 1)
                .unwrap(),
            9
        );
        let zone = state.zones.get("spawn").unwrap();
        assert!(zone.chunk_aligned);
        assert_eq!(
            (zone.min_x, zone.min_z, zone.max_x, zone.max_z),
            (0, 0, 47, 47)
        );
    }
    #[test]
    fn global_war_slot_tracks_live_lifecycle() {
        let mut s = FactionState::default();
        let mut war = War {
            id: "war".into(),
            attacker: "a".into(),
            defender: "b".into(),
            forced: false,
            status: WarStatus::Requested,
            requested_at: 1,
            request_expires_at: 2,
            preparation_ends_at: None,
            battle_ends_at: None,
            ready: Default::default(),
            prisoners: Default::default(),
            winner: None,
            loser: None,
            reparations: 0,
            arena_id: "default".into(),
        };
        s.wars.insert("war".into(), war.clone());
        assert!(s.war_slot_busy());
        war.status = WarStatus::Finished;
        s.wars.insert("war".into(), war);
        assert!(!s.war_slot_busy());
    }

    #[test]
    fn legacy_arena_migrates_to_named_spawn_groups() {
        let mut state = FactionState {
            schema_version: 2,
            arena: Some(Location {
                world: "world".into(),
                x: 1.0,
                y: 2.0,
                z: 3.0,
            }),
            arena_team2: Some(Location {
                world: "world".into(),
                x: 4.0,
                y: 5.0,
                z: 6.0,
            }),
            ..Default::default()
        };
        state.migrate();
        assert_eq!(state.schema_version, SCHEMA_VERSION);
        assert_eq!(state.arenas["default"].team1_spawns.len(), 1);
        assert_eq!(state.arenas["default"].team2_spawns.len(), 1);
    }

    #[test]
    fn pre_v04_faction_keeps_data_but_requires_a_physical_core() {
        let mut state = FactionState::default();
        let id = state
            .create("Legacy", "alice", Visibility::Public, 1, 10)
            .unwrap();
        establish(&mut state, &id, 0);
        state.factions.get_mut(&id).unwrap().bank = 500;
        state.schema_version = 3;
        state.migrate();
        let faction = &state.factions[&id];
        assert_eq!(faction.bank, 500);
        assert_eq!(faction.members.get("alice"), Some(&Role::Leader));
        assert!(faction.claims.is_empty());
        assert_eq!(
            faction.last_claim_snapshot.as_ref().unwrap().claims.len(),
            9
        );
        assert_eq!(faction.core_lifecycle, CoreLifecycle::AwaitingCore);
        assert!(faction.physical_core.is_none());
        assert!(state.core_owners.is_empty());
    }

    #[test]
    fn arenas_rotate_deterministically() {
        let mut state = FactionState::default();
        let location = Location {
            world: "world".into(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
        };
        for id in ["bravo", "alpha"] {
            state.arenas.insert(
                id.into(),
                Arena {
                    id: id.into(),
                    team1_spawns: vec![location.clone()],
                    team2_spawns: vec![location.clone()],
                    enabled: true,
                },
            );
        }
        assert_eq!(state.select_arena().unwrap(), "alpha");
        assert_eq!(state.select_arena().unwrap(), "bravo");
        assert_eq!(state.select_arena().unwrap(), "alpha");
    }

    #[test]
    fn safe_zone_takes_precedence_over_overlapping_war_zone() {
        let mut state = FactionState::default();
        let first = Location {
            world: "world".into(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let second = Location {
            world: "world".into(),
            x: 20.0,
            y: 0.0,
            z: 20.0,
        };
        state
            .set_zone("war", ZoneKind::War, &first, &second)
            .unwrap();
        state
            .set_zone("safe", ZoneKind::Safe, &first, &second)
            .unwrap();
        assert_eq!(state.zone_at("world", 10, 10).unwrap().kind, ZoneKind::Safe);
        assert!(state.environmental_protected(Some("world"), 10, 10));
    }

    #[test]
    fn war_policy_reports_explicit_shield_and_grace_reasons() {
        let mut state = FactionState::default();
        state
            .create("Alpha", "a", Visibility::Public, 1, 10)
            .unwrap();
        state
            .create("Bravo", "b", Visibility::Public, 1, 10)
            .unwrap();
        state
            .factions
            .get_mut("bravo")
            .unwrap()
            .war_policy
            .shield_until = 100;
        assert!(
            state
                .war_block_reason("alpha", "bravo", 50)
                .unwrap()
                .contains("shield")
        );
        state
            .factions
            .get_mut("bravo")
            .unwrap()
            .war_policy
            .shield_until = 0;
        state
            .factions
            .get_mut("bravo")
            .unwrap()
            .war_policy
            .grace_until = 100;
        assert!(
            state
                .war_block_reason("alpha", "bravo", 50)
                .unwrap()
                .contains("grace")
        );
    }

    #[test]
    fn trade_components_survive_json_round_trip() {
        let item = TradeItem {
            registry_key: "minecraft:diamond_sword".into(),
            count: 1,
            components: vec![TradeComponent {
                id: 13,
                value: vec![1, 2, 3, 4],
            }],
        };
        let json = serde_json::to_string(&item).unwrap();
        let restored: TradeItem = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.components, item.components);
    }

    #[test]
    fn claim_lookup_load_fixture_covers_ten_thousand_claims() {
        let mut state = FactionState::default();
        for faction_index in 0..100 {
            let name = format!("f{faction_index:03}");
            let leader = format!("p{faction_index:03}");
            let id = state
                .create(&name, &leader, Visibility::Public, 1, 10_000)
                .unwrap();
            let faction = state.factions.get_mut(&id).unwrap();
            faction.core_lifecycle = CoreLifecycle::Active;
            faction.physical_core = Some(PhysicalCore {
                location: BlockLocation {
                    world: "world".into(),
                    x: faction_index * 100 * 16,
                    y: 64,
                    z: faction_index * 16,
                },
                lives: 10,
                max_lives: 10,
                last_hit_at: 0,
                established_at: 1,
            });
            for claim_index in 0..100 {
                faction.claims.insert(Claim {
                    world: "world".into(),
                    chunk_x: faction_index * 100 + claim_index,
                    chunk_z: faction_index,
                });
            }
        }
        state.rebuild_claim_index();
        for faction_index in 0..100 {
            for claim_index in 0..100 {
                let claim = Claim {
                    world: "world".into(),
                    chunk_x: faction_index * 100 + claim_index,
                    chunk_z: faction_index,
                };
                assert!(state.claim_owner(&claim).is_some());
            }
        }
    }
}
