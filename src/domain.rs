use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const SCHEMA_VERSION: u32 = 2;

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
    pub audit: Vec<AuditEvent>,
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
            audit: vec![],
        }
    }
}

impl FactionState {
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
                created_at: now,
            },
        );
        self.player_faction.insert(leader.into(), id.clone());
        Ok(id)
    }
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let faction = self.factions.remove(id).ok_or("faction not found")?;
        for p in faction.members.keys() {
            self.player_faction.remove(p);
        }
        self.relations.retain(|k, _| !k.split('|').any(|v| v == id));
        self.applications.retain(|a| a.faction != id);
        self.invitations.retain(|i| i.faction != id);
        Ok(())
    }
    pub fn invite(&mut self, faction: &str, player: &str, expires_at: u64) -> Result<(), String> {
        if !self.factions.contains_key(faction) {
            return Err("faction not found".into());
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
        if self.factions.values().any(|f| f.claims.contains(&claim)) {
            return Err("chunk is already claimed".into());
        }
        let f = self.factions.get_mut(id).ok_or("faction not found")?;
        if f.claims.len() >= f.power.max(0) as usize {
            return Err("not enough faction power".into());
        }
        f.claims.insert(claim);
        Ok(())
    }
    pub fn overclaim(&mut self, attacker: &str, claim: Claim) -> Result<String, String> {
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
        if attacking.claims.len() >= attacking.power.max(0) as usize {
            return Err("your faction lacks power for another claim".into());
        }
        self.factions.get_mut(&owner).unwrap().claims.remove(&claim);
        self.factions
            .get_mut(attacker)
            .unwrap()
            .claims
            .insert(claim);
        Ok(owner)
    }
    pub fn claim_owner(&self, claim: &Claim) -> Option<&Faction> {
        self.factions.values().find(|f| f.claims.contains(claim))
    }
    pub fn overclaimed(&self, id: &str) -> bool {
        self.factions
            .get(id)
            .is_some_and(|f| f.claims.len() > f.power.max(0) as usize)
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
    #[test]
    fn lifecycle() {
        let mut s = FactionState::default();
        let id = s
            .create("Knights", "alice", Visibility::Private, 1, 10)
            .unwrap();
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
        s.create("Knights", "alice", Visibility::Public, 1, 1)
            .unwrap();
        s.claim(
            "knights",
            Claim {
                world: "world".into(),
                chunk_x: 0,
                chunk_z: 0,
            },
        )
        .unwrap();
        assert!(
            s.claim(
                "knights",
                Claim {
                    world: "world".into(),
                    chunk_x: 1,
                    chunk_z: 0
                }
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
        let claim = Claim {
            world: "world".into(),
            chunk_x: 0,
            chunk_z: 0,
        };
        s.claim("aaa", claim.clone()).unwrap();
        s.factions.get_mut("aaa").unwrap().power = 0;
        s.set_relation("aaa", "bbb", Relation::Enemy).unwrap();
        assert_eq!(s.overclaim("bbb", claim).unwrap(), "aaa");
    }
}
