use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Leader,
    Officer,
    Veteran,
    Member,
    Recruit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Relation {
    Neutral,
    Truce,
    Ally,
    Enemy,
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
    pub bank: i64,
    pub prison: Option<PrisonLocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Claim {
    pub world: String,
    pub chunk_x: i32,
    pub chunk_z: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrisonLocation {
    pub world: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct War {
    pub id: String,
    pub attacker: String,
    pub defender: String,
    pub forced: bool,
    pub status: WarStatus,
    pub ready: HashSet<String>,
    pub prisoners: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarStatus {
    Requested,
    Preparing,
    Active,
    Finished { winner: String },
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FactionState {
    pub factions: HashMap<String, Faction>,
    pub player_faction: HashMap<String, String>,
    pub relations: HashMap<(String, String), Relation>,
    pub wars: HashMap<String, War>,
    pub mail: HashMap<String, Vec<String>>,
    pub trade: HashMap<(String, String), Vec<String>>,
}

impl FactionState {
    pub fn create(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        leader: impl Into<String>,
        visibility: Visibility,
    ) -> Result<(), String> {
        let id = id.into();
        let leader = leader.into();
        if self.factions.contains_key(&id) {
            return Err("faction id already exists".into());
        }
        if self.player_faction.contains_key(&leader) {
            return Err("leader already belongs to a faction".into());
        }
        let mut members = HashMap::new();
        members.insert(leader.clone(), Role::Leader);
        self.player_faction.insert(leader.clone(), id.clone());
        self.factions.insert(
            id.clone(),
            Faction {
                id,
                name: name.into(),
                description: String::new(),
                visibility,
                leader,
                members,
                claims: HashSet::new(),
                power: 10,
                bank: 0,
                prison: None,
            },
        );
        Ok(())
    }
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let faction = self.factions.remove(id).ok_or("faction not found")?;
        for player in faction.members.keys() {
            self.player_faction.remove(player);
        }
        self.relations.retain(|(a, b), _| a != id && b != id);
        Ok(())
    }
    pub fn invite(&mut self, faction: &str, player: &str) -> Result<(), String> {
        if !self.factions.contains_key(faction) {
            return Err("faction not found".into());
        }
        if self.player_faction.contains_key(player) {
            return Err("player already belongs to a faction".into());
        }
        self.mail
            .entry(player.into())
            .or_default()
            .push(format!("invite:{faction}"));
        Ok(())
    }
    pub fn apply(&mut self, faction: &str, player: &str) -> Result<(), String> {
        let f = self.factions.get(faction).ok_or("faction not found")?;
        if f.visibility == Visibility::Private {
            return Err("faction is invitation-only".into());
        }
        if self.player_faction.contains_key(player) {
            return Err("player already belongs to a faction".into());
        }
        self.mail
            .entry(f.leader.clone())
            .or_default()
            .push(format!("application:{faction}:{player}"));
        Ok(())
    }
    pub fn accept_member(&mut self, faction: &str, player: &str) -> Result<(), String> {
        if self.player_faction.contains_key(player) {
            return Err("player already belongs to a faction".into());
        }
        let f = self.factions.get_mut(faction).ok_or("faction not found")?;
        f.members.insert(player.into(), Role::Recruit);
        self.player_faction.insert(player.into(), faction.into());
        Ok(())
    }
    pub fn set_relation(&mut self, a: &str, b: &str, relation: Relation) -> Result<(), String> {
        if !self.factions.contains_key(a) || !self.factions.contains_key(b) {
            return Err("faction not found".into());
        }
        self.relations
            .insert((a.into(), b.into()), relation.clone());
        self.relations.insert((b.into(), a.into()), relation);
        Ok(())
    }
    pub fn faction_of(&self, player: &str) -> Option<&Faction> {
        self.player_faction
            .get(player)
            .and_then(|id| self.factions.get(id))
    }
    pub fn claim(&mut self, faction: &str, claim: Claim) -> Result<(), String> {
        let f = self.factions.get_mut(faction).ok_or("faction not found")?;
        let limit = f.power.max(0) as usize;
        if !f.claims.contains(&claim) && f.claims.len() >= limit {
            return Err("claim exceeds available power".into());
        }
        f.claims.insert(claim);
        Ok(())
    }
    pub fn overclaimed(&self, faction: &str) -> bool {
        self.factions
            .get(faction)
            .is_some_and(|f| f.claims.len() > f.power.max(0) as usize)
    }
    pub fn set_prison(&mut self, faction: &str, prison: PrisonLocation) -> Result<(), String> {
        self.factions
            .get_mut(faction)
            .ok_or("faction not found")?
            .prison = Some(prison);
        Ok(())
    }
    pub fn start_war(
        &mut self,
        id: impl Into<String>,
        attacker: &str,
        defender: &str,
        forced: bool,
    ) -> Result<(), String> {
        if !self.factions.contains_key(attacker) || !self.factions.contains_key(defender) {
            return Err("faction not found".into());
        }
        if !forced
            && self
                .factions
                .get(attacker)
                .and_then(|f| f.prison.as_ref())
                .is_none()
        {
            return Err("attacker must set a prison before war".into());
        }
        let id = id.into();
        self.wars.insert(
            id.clone(),
            War {
                id,
                attacker: attacker.into(),
                defender: defender.into(),
                forced,
                status: if forced {
                    WarStatus::Preparing
                } else {
                    WarStatus::Requested
                },
                ready: HashSet::new(),
                prisoners: HashMap::new(),
            },
        );
        Ok(())
    }
}

pub trait FactionLookup {
    fn faction_id(&self, player: &str) -> Option<&str>;
    fn faction_name(&self, player: &str) -> Option<&str>;
}
impl FactionLookup for FactionState {
    fn faction_id(&self, player: &str) -> Option<&str> {
        self.player_faction.get(player).map(String::as_str)
    }
    fn faction_name(&self, player: &str) -> Option<&str> {
        self.faction_of(player).map(|f| f.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_and_private_join_rules() {
        let mut s = FactionState::default();
        s.create("a", "A", "alice", Visibility::Private).unwrap();
        assert!(s.apply("a", "bob").is_err());
        s.factions.get_mut("a").unwrap().visibility = Visibility::Public;
        s.apply("a", "bob").unwrap();
    }
    #[test]
    fn claims_respect_power_and_overclaim() {
        let mut s = FactionState::default();
        s.create("a", "A", "alice", Visibility::Public).unwrap();
        s.claim(
            "a",
            Claim {
                world: "w".into(),
                chunk_x: 0,
                chunk_z: 0,
            },
        )
        .unwrap();
        assert!(!s.overclaimed("a"));
    }
    #[test]
    fn api_resolves_membership() {
        let mut s = FactionState::default();
        s.create("a", "A", "alice", Visibility::Public).unwrap();
        assert_eq!(s.faction_name("alice"), Some("A"));
    }
}
