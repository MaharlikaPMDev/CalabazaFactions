mod app;
mod config;
pub mod domain;
mod storage;
mod ui;

use app::{App, TradeView};
use domain::*;
use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, Server,
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    events::{
        BedrockFormResponseEvent, BlockBreakEvent, BlockPlaceEvent, EntityDamageByEntityEvent,
        EventData, EventHandler, EventPriority, InventoryClickEvent, InventoryCloseEvent,
        PlayerDeathEvent, PlayerJoinEvent, PlayerLeaveEvent, PlayerMoveEvent,
    },
    forms::FormResponse,
    permission::{Permission, PermissionDefault, PermissionLevel},
    permissions,
    text::TextComponent,
};
use std::{collections::HashSet, path::PathBuf, sync::Arc};

const PERM_USER: &str = "CalabazaFactions:command.faction";
const PERM_ADMIN: &str = "CalabazaFactions:command.admin";
struct CalabazaFactions;

impl Plugin for CalabazaFactions {
    fn new() -> Self {
        Self
    }
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata{name:"CalabazaFactions".into(),version:env!("CARGO_PKG_VERSION").into(),authors:vec!["MaharlikaPMDev".into()],description:"Playable factions, claims, diplomacy, wars, POWs, economy, mail, and alliance trade for PumpkinMC.".into(),dependencies:vec![],permissions:vec![permissions::FS_READ_DATA.into(),permissions::FS_WRITE_DATA.into()]}
    }
    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        let app = Arc::new(App::load(PathBuf::from(context.get_data_folder()))?);
        context.register_permission(&Permission {
            node: PERM_USER.into(),
            description: "Use CalabazaFactions".into(),
            default: PermissionDefault::Allow,
            children: vec![],
        })?;
        context.register_permission(&Permission {
            node: PERM_ADMIN.into(),
            description: "Administer CalabazaFactions".into(),
            default: PermissionDefault::Op(PermissionLevel::Three),
            children: vec![],
        })?;
        register_command(&context, app.clone());
        context.register_event_handler(Join(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(Leave(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(Break(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(Place(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(Damage(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(Death(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(Move(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(Click(app.clone()), EventPriority::Highest, true)?;
        context.register_event_handler(Close(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(Form(app), EventPriority::Normal, true)?;
        tracing::info!(
            "CalabazaFactions v{} loaded without a scheduler",
            env!("CARGO_PKG_VERSION")
        );
        Ok(())
    }
}
pumpkin_plugin_api::register_plugin!(CalabazaFactions);

fn register_command(context: &Context, app: Arc<App>) {
    let handler = FactionCommand { app: app.clone() };
    let command = Command::new(
        &["faction".into(), "f".into()],
        "CalabazaFactions main command",
    )
    .execute(FactionCommand { app: app.clone() })
    .then(
        CommandNode::argument("input", &ArgumentType::String(StringType::Greedy)).execute(handler),
    );
    context.register_command(command, PERM_USER);
}

struct FactionCommand {
    app: Arc<App>,
}
impl CommandHandler for FactionCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let Some(player) = sender.as_player() else {
            return Err(fail("This command must be used in game."));
        };
        self.app.remember_player(&player);
        process_wars(&self.app, &server);
        let input = match args.get_value("input") {
            Arg::Simple(v) | Arg::Msg(v) => v,
            _ => String::new(),
        };
        match execute(
            &self.app,
            &server,
            &player,
            &input,
            sender.has_permission(&server, PERM_ADMIN),
        ) {
            Ok(message) => {
                if !message.is_empty() {
                    player.send_system_message(TextComponent::text(&message), false);
                }
                Ok(1)
            }
            Err(error) => Err(fail(&error)),
        }
    }
}
fn fail(message: &str) -> CommandError {
    CommandError::CommandFailed(TextComponent::text(message))
}
fn pid(p: &pumpkin_plugin_api::Player) -> String {
    App::player_id(p)
}
fn resolve_player(app: &App, server: &Server, name: &str) -> Option<String> {
    server
        .get_player_by_name(name)
        .map(|p| pid(&p))
        .or_else(|| {
            let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
            s.player_names
                .iter()
                .find(|(_, n)| n.eq_ignore_ascii_case(name))
                .map(|(id, _)| id.clone())
        })
}
fn require_faction(s: &FactionState, p: &str) -> Result<String, String> {
    s.player_faction
        .get(p)
        .cloned()
        .ok_or_else(|| "you are not in a faction".into())
}
fn require_manager(s: &FactionState, p: &str) -> Result<String, String> {
    let id = require_faction(s, p)?;
    if !s.role_of(p).is_some_and(Role::can_manage) {
        return Err("leader or officer role required".into());
    }
    Ok(id)
}
fn require_leader(s: &FactionState, p: &str) -> Result<String, String> {
    let id = require_faction(s, p)?;
    if s.factions.get(&id).is_none_or(|f| f.leader != p) {
        return Err("faction leader required".into());
    }
    Ok(id)
}

fn execute(
    app: &App,
    server: &Server,
    player: &pumpkin_plugin_api::Player,
    input: &str,
    is_admin: bool,
) -> Result<String, String> {
    let words = input.split_whitespace().collect::<Vec<_>>();
    let action = words
        .first()
        .copied()
        .unwrap_or("menu")
        .to_ascii_lowercase();
    let player_id = pid(player);
    let now = App::now();
    match action.as_str(){
        "menu"|"page"=>{ui::open_faction(app,player);Ok(String::new())},
        "help"=>Ok("/faction create <name> [public|private], disband, invite/apply/join/accept, leave/kick/role/transfer, public/private, claim/unclaim/overclaim/map, sethome/home, bank, relation, setprison, war/forcewar/waraccept/wardecline/ready, paypow, mail, trade/tradeinbox".into()),
        "create"=>{let name=*words.get(1).ok_or("usage: /faction create <name> [public|private]")?;let vis=if words.get(2).is_some_and(|v|v.eq_ignore_ascii_case("private")){Visibility::Private}else{Visibility::Public};let id=app.mutate(&player_id,"create",|s|s.create(name,&player_id,vis,now,app.config.factions.starting_power))?;Ok(format!("Faction {id} created."))},
        "disband"=>{app.mutate(&player_id,"disband",|s|{let id=require_leader(s,&player_id)?;s.delete(&id)})?;Ok("Faction disbanded.".into())},
        "invite"=>{let name=*words.get(1).ok_or("usage: /faction invite <player>")?;let target=resolve_player(app,server,name).ok_or("player must have joined this server before")?;app.mutate(&player_id,"invite",|s|{let id=require_manager(s,&player_id)?;s.invite(&id,&target,now+72*3600)?;s.send_mail(&id,"Invitation sent",&format!("{} invited {name}",s.player_names.get(&player_id).cloned().unwrap_or_default()),now);Ok(())})?;Ok(format!("Invited {name}."))},
        "apply"=>{let faction=FactionState::normalize(words.get(1).ok_or("usage: /faction apply <faction>")?);app.mutate(&player_id,"apply",|s|{s.apply(&faction,&player_id,now)?;s.send_mail(&faction,"New application",&format!("{} applied to join.",s.player_names.get(&player_id).cloned().unwrap_or(player_id.clone())),now);Ok(())})?;Ok("Application sent through Faction Mail.".into())},
        "join"=>{let faction=FactionState::normalize(words.get(1).ok_or("usage: /faction join <faction>")?);app.mutate(&player_id,"join",|s|s.join(&faction,&player_id,now,app.config.factions.max_members,true))?;Ok(format!("Joined {faction}."))},
        "accept"=>{let name=*words.get(1).ok_or("usage: /faction accept <player>")?;let target=resolve_player(app,server,name).ok_or("unknown player")?;app.mutate(&player_id,"accept",|s|{let id=require_manager(s,&player_id)?;if !s.applications.iter().any(|a|a.faction==id&&a.player==target){return Err("no application from that player".into())}s.join(&id,&target,now,app.config.factions.max_members,false)})?;Ok(format!("Accepted {name}."))},
        "leave"=>{app.mutate(&player_id,"leave",|s|s.leave(&player_id))?;Ok("You left your faction.".into())},
        "kick"=>{let name=*words.get(1).ok_or("usage: /faction kick <player>")?;let target=resolve_player(app,server,name).ok_or("unknown player")?;app.mutate(&player_id,"kick",|s|{let id=require_manager(s,&player_id)?;if s.factions[&id].leader==target{return Err("cannot kick the leader".into())}s.factions.get_mut(&id).ok_or("faction not found")?.members.remove(&target).ok_or("player is not a member")?;s.player_faction.remove(&target);Ok(())})?;Ok(format!("Kicked {name}."))},
        "role"=>{let name=*words.get(1).ok_or("usage: /faction role <player> <officer|veteran|member|recruit>")?;let role=Role::parse(words.get(2).ok_or("missing role")?).ok_or("invalid role")?;if role==Role::Leader{return Err("use /faction transfer".into())}let target=resolve_player(app,server,name).ok_or("unknown player")?;app.mutate(&player_id,"role",|s|{let id=require_leader(s,&player_id)?;s.factions.get_mut(&id).unwrap().members.get_mut(&target).map(|r|*r=role).ok_or_else(||"player is not a member".to_string())})?;Ok(format!("Updated {name}'s role."))},
        "transfer"=>{let name=*words.get(1).ok_or("usage: /faction transfer <player>")?;let target=resolve_player(app,server,name).ok_or("unknown player")?;app.mutate(&player_id,"transfer",|s|{let id=require_leader(s,&player_id)?;let f=s.factions.get_mut(&id).unwrap();if !f.members.contains_key(&target){return Err("player is not a member".into())}f.members.insert(player_id.clone(),Role::Officer);f.members.insert(target.clone(),Role::Leader);f.leader=target.clone();Ok(())})?;Ok(format!("Leadership transferred to {name}."))},
        "public"|"private"=>{let vis=if action=="public"{Visibility::Public}else{Visibility::Private};app.mutate(&player_id,"visibility",|s|{let id=require_manager(s,&player_id)?;s.factions.get_mut(&id).unwrap().visibility=vis;Ok(())})?;Ok(format!("Faction is now {action}."))},
        "claim"=>{let claim=App::claim_at(player);app.mutate(&player_id,"claim",|s|{let id=require_manager(s,&player_id)?;s.claim(&id,claim)})?;Ok("Chunk claimed.".into())},
        "overclaim"=>{let claim=App::claim_at(player);let previous=app.mutate(&player_id,"overclaim",|s|{let id=require_manager(s,&player_id)?;s.overclaim(&id,claim)})?;Ok(format!("Overclaimed this chunk from {previous}."))},
        "unclaim"=>{let claim=App::claim_at(player);app.mutate(&player_id,"unclaim",|s|{let id=require_manager(s,&player_id)?;if !s.factions.get_mut(&id).unwrap().claims.remove(&claim){return Err("your faction does not own this chunk".into())}Ok(())})?;Ok("Chunk unclaimed.".into())},
        "map"=>{let claim=App::claim_at(player);let s=app.state.lock().unwrap_or_else(|e|e.into_inner());let mut out=String::from("Nearby claims:\n");for z in -2..=2{for x in -2..=2{let c=Claim{world:claim.world.clone(),chunk_x:claim.chunk_x+x,chunk_z:claim.chunk_z+z};out.push(if x==0&&z==0{'@'}else if s.claim_owner(&c).is_some(){'#'}else{'.'});}out.push('\n');}Ok(out)},
        "sethome"|"setprison"=>{let loc=App::location(player);app.mutate(&player_id,&action,|s|{let id=require_manager(s,&player_id)?;let f=s.factions.get_mut(&id).unwrap();if action=="sethome"{f.home=Some(loc)}else{f.prison=Some(loc)}Ok(())})?;Ok(format!("Faction {} set.",if action=="sethome"{"home"}else{"prison"}))},
        "home"=>{let s=app.state.lock().unwrap_or_else(|e|e.into_inner());let id=require_faction(&s,&player_id)?;let loc=s.factions[&id].home.clone().ok_or("faction home is not set")?;drop(s);teleport(server,player,&loc)?;Ok("Teleported to faction home.".into())},
        "bank"=>bank_command(app,&player_id,&words),
        "relation"=>{let target=FactionState::normalize(words.get(1).ok_or("usage: /faction relation <faction> <neutral|truce|ally|enemy>")?);let relation=Relation::parse(words.get(2).ok_or("missing relation")?).ok_or("invalid relation")?;app.mutate(&player_id,"relation",|s|{let id=require_manager(s,&player_id)?;s.set_relation(&id,&target,relation.clone())?;s.send_mail(&target,"Diplomacy updated",&format!("{id} set relations to {relation:?}."),now);Ok(())})?;Ok(format!("Relation with {target}: {relation:?}."))},
        "setarena"=>{if !is_admin{return Err("admin permission required".into())}let loc=App::location(player);app.mutate(&player_id,"setarena",|s|{s.arena=Some(loc);Ok(())})?;Ok("War arena set.".into())},
        "war"|"forcewar"=>start_war(app,&player_id,words.get(1).ok_or("usage: /faction war <faction>")?,action=="forcewar",now),
        "waraccept"|"wardecline"=>answer_war(app,&player_id,action=="waraccept",now),
        "ready"=>ready_war(app,server,&player_id,now),
        "paypow"=>pay_pow(app,&player_id,words.get(1).ok_or("usage: /faction paypow <player>")?,server,now),
        "mail"=>{ui::open_mail(app,player);Ok(String::new())},
        "trade"=>{let target=FactionState::normalize(words.get(1).ok_or("usage: /faction trade <allied faction>")?);ui::open_trade_send(app,player,&target)?;Ok(String::new())},
        "tradeinbox"=>{ui::open_trade_inbox(app,player)?;Ok(String::new())},
        _=>Err("Unknown subcommand. Use /faction help".into()),
    }
}

fn bank_command(app: &App, p: &str, words: &[&str]) -> Result<String, String> {
    let op = words.get(1).copied().unwrap_or("balance");
    app.mutate(p, "bank", |s| {
        let id = require_faction(s, p)?;
        match op {
            "balance" => Ok(format!(
                "Faction bank: {} • Wallet: {}",
                s.factions[&id].bank,
                s.wallets.get(p).copied().unwrap_or(0)
            )),
            "deposit" => {
                let n = words
                    .get(2)
                    .ok_or("usage: /faction bank deposit <amount>")?
                    .parse::<i64>()
                    .map_err(|_| "invalid amount")?;
                if n <= 0 || s.wallets.get(p).copied().unwrap_or(0) < n {
                    return Err("insufficient wallet balance".into());
                }
                *s.wallets.entry(p.into()).or_default() -= n;
                s.factions.get_mut(&id).unwrap().bank += n;
                Ok(format!("Deposited {n}."))
            }
            "withdraw" => {
                if !s.role_of(p).is_some_and(Role::can_manage) {
                    return Err("leader or officer required".into());
                }
                let n = words
                    .get(2)
                    .ok_or("usage: /faction bank withdraw <amount>")?
                    .parse::<i64>()
                    .map_err(|_| "invalid amount")?;
                if n <= 0 || s.factions[&id].bank < n {
                    return Err("insufficient faction balance".into());
                }
                s.factions.get_mut(&id).unwrap().bank -= n;
                *s.wallets.entry(p.into()).or_default() += n;
                Ok(format!("Withdrew {n}."))
            }
            _ => Err("bank options: balance, deposit, withdraw".into()),
        }
    })
}

fn start_war(app: &App, p: &str, target: &str, forced: bool, now: u64) -> Result<String, String> {
    let target = FactionState::normalize(target);
    app.mutate(p, "war", |s| {
        let attacker = require_leader(s, p)?;
        if !s.factions.contains_key(&target) {
            return Err("target faction not found".into());
        }
        if s.factions[&attacker].prison.is_none() || s.factions[&target].prison.is_none() {
            return Err("both factions must set a prison before war".into());
        }
        if !forced && s.relation(&attacker, &target) != Relation::Enemy {
            return Err("declare the target an enemy first".into());
        }
        let id = format!("{attacker}-{target}-{now}");
        let status = if forced {
            WarStatus::Preparing
        } else {
            WarStatus::Requested
        };
        s.wars.insert(
            id.clone(),
            War {
                id: id.clone(),
                attacker: attacker.clone(),
                defender: target.clone(),
                forced,
                status,
                requested_at: now,
                request_expires_at: now + app.config.war.request_hours * 3600,
                preparation_ends_at: forced
                    .then_some(now + app.config.war.preparation_hours * 3600),
                battle_ends_at: None,
                ready: HashSet::new(),
                prisoners: Default::default(),
                winner: None,
                loser: None,
                reparations: 0,
            },
        );
        s.send_mail(
            &target,
            if forced {
                "Forced war declared"
            } else {
                "War request"
            },
            &format!(
                "{attacker} challenged your faction. Request expires in {} hours.",
                app.config.war.request_hours
            ),
            now,
        );
        Ok(())
    })?;
    Ok(if forced {
        "Forced war scheduled after preparation.".into()
    } else {
        "War request delivered by Faction Mail.".into()
    })
}
fn answer_war(app: &App, p: &str, accept: bool, now: u64) -> Result<String, String> {
    app.mutate(p, "war-answer", |s| {
        let faction = require_leader(s, p)?;
        let war = s
            .wars
            .values_mut()
            .filter(|w| {
                w.defender == faction
                    && w.status == WarStatus::Requested
                    && w.request_expires_at >= now
            })
            .max_by_key(|w| w.requested_at)
            .ok_or("no pending war request")?;
        if accept {
            war.status = WarStatus::Preparing;
            war.preparation_ends_at = Some(now + app.config.war.preparation_hours * 3600)
        } else {
            war.status = WarStatus::Declined
        }
        Ok(())
    })?;
    Ok(if accept {
        "War accepted; preparation has begun.".into()
    } else {
        "War declined.".into()
    })
}
fn ready_war(app: &App, server: &Server, p: &str, now: u64) -> Result<String, String> {
    let should_start = app.mutate(p, "war-ready", |s| {
        let faction = require_leader(s, p)?;
        let war = s
            .wars
            .values_mut()
            .find(|w| {
                w.status == WarStatus::Preparing && (w.attacker == faction || w.defender == faction)
            })
            .ok_or("no preparing war")?;
        war.ready.insert(faction);
        if war.ready.len() == 2 {
            war.status = WarStatus::Active;
            war.battle_ends_at = Some(now + app.config.war.battle_minutes * 60);
            Ok(true)
        } else {
            war.preparation_ends_at = Some(
                war.preparation_ends_at
                    .unwrap_or(u64::MAX)
                    .min(now + app.config.war.ready_countdown_minutes * 60),
            );
            Ok(false)
        }
    })?;
    if should_start {
        teleport_active_wars(app, server);
        Ok("Both leaders are ready. War started immediately!".into())
    } else {
        Ok("Ready marked. Countdown shortened to five minutes.".into())
    }
}
fn pay_pow(app: &App, p: &str, name: &str, server: &Server, now: u64) -> Result<String, String> {
    let target = resolve_player(app, server, name).ok_or("unknown player")?;
    app.mutate(p, "pay-pow", |s| {
        let faction = require_manager(s, p)?;
        let (war_id, ransom, captor) = s
            .wars
            .iter()
            .find_map(|(id, w)| {
                w.prisoners
                    .get(&target)
                    .filter(|_| w.attacker == faction || w.defender == faction)
                    .map(|pr| (id.clone(), pr.ransom, pr.captor_faction.clone()))
            })
            .ok_or("player is not a POW")?;
        if s.factions[&faction].bank < ransom {
            return Err("insufficient faction bank balance".into());
        }
        s.factions.get_mut(&faction).unwrap().bank -= ransom;
        s.factions.get_mut(&captor).unwrap().bank += ransom;
        s.wars.get_mut(&war_id).unwrap().prisoners.remove(&target);
        s.send_mail(
            &faction,
            "POW released",
            &format!("{name} released for {ransom}."),
            now,
        );
        Ok(())
    })?;
    Ok(format!("Paid ransom and released {name}."))
}
fn teleport(server: &Server, p: &pumpkin_plugin_api::Player, loc: &Location) -> Result<(), String> {
    let world = server
        .get_world_by_name(&loc.world)
        .ok_or("configured world is unavailable")?;
    p.teleport((loc.x, loc.y, loc.z), None, None, world);
    Ok(())
}

fn process_wars(app: &App, server: &Server) {
    let now = App::now();
    let changed = {
        let mut s = app.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut changed = false;
        let timed_out = s
            .wars
            .values()
            .filter(|w| w.status == WarStatus::Active && w.battle_ends_at.is_some_and(|v| v <= now))
            .map(|w| (w.id.clone(), w.defender.clone(), w.attacker.clone()))
            .collect::<Vec<_>>();
        for w in s.wars.values_mut() {
            match w.status {
                WarStatus::Requested if w.request_expires_at <= now => {
                    w.status = WarStatus::Expired;
                    changed = true
                }
                WarStatus::Preparing if w.preparation_ends_at.is_some_and(|v| v <= now) => {
                    w.status = WarStatus::Active;
                    w.battle_ends_at = Some(now + app.config.war.battle_minutes * 60);
                    changed = true
                }
                _ => {}
            }
            let before = w.prisoners.len();
            w.prisoners.retain(|_, prisoner| prisoner.release_at > now);
            changed |= before != w.prisoners.len();
        }
        for (id, winner, loser) in timed_out {
            finish_war(&mut s, &id, &winner, &loser, &app.config);
            changed = true;
        }
        if changed {
            let _ = storage::save(&app.data_dir, &s);
        }
        changed
    };
    if changed {
        teleport_active_wars(app, server)
    }
}
fn teleport_active_wars(app: &App, server: &Server) {
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let Some(arena) = s.arena.clone() else { return };
    let participants = s
        .wars
        .values()
        .filter(|w| w.status == WarStatus::Active)
        .flat_map(|w| [&w.attacker, &w.defender])
        .flat_map(|id| s.factions.get(id))
        .flat_map(|f| f.members.keys())
        .cloned()
        .collect::<HashSet<_>>();
    drop(s);
    for p in server.get_all_players() {
        if participants.contains(&pid(&p)) {
            let _ = teleport(server, &p, &arena);
            p.send_system_message(TextComponent::text("The faction war has begun. Attackers have 30 minutes to defeat the defending leader."),false);
        }
    }
}

struct Join(Arc<App>);
impl EventHandler<PlayerJoinEvent> for Join {
    fn handle(
        &self,
        server: Server,
        event: EventData<PlayerJoinEvent>,
    ) -> EventData<PlayerJoinEvent> {
        self.0.remember_player(&event.player);
        process_wars(&self.0, &server);
        let id = pid(&event.player);
        if let Some(n) = self.0.war_notice(&id) {
            event
                .player
                .send_system_message(TextComponent::text(&n), false)
        }
        let prison = {
            let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            s.wars.values().find_map(|w| {
                w.prisoners
                    .get(&id)
                    .filter(|p| p.release_at > App::now())
                    .and_then(|p| s.factions.get(&p.captor_faction)?.prison.clone())
            })
        };
        if let Some(loc) = prison {
            let _ = teleport(&server, &event.player, &loc);
            event.player.send_system_message(TextComponent::text("You are a POW. Your faction may pay ransom, otherwise release occurs after 24 hours."),false);
        }
        event
    }
}
struct Leave(Arc<App>);
impl EventHandler<PlayerLeaveEvent> for Leave {
    fn handle(&self, _: Server, event: EventData<PlayerLeaveEvent>) -> EventData<PlayerLeaveEvent> {
        let id = pid(&event.player);
        self.0
            .menus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        event
    }
}
struct Move(Arc<App>);
impl EventHandler<PlayerMoveEvent> for Move {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<PlayerMoveEvent>,
    ) -> EventData<PlayerMoveEvent> {
        let id = pid(&event.player);
        let prison = {
            let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            s.wars.values().find_map(|w| {
                w.prisoners
                    .get(&id)
                    .filter(|p| p.release_at > App::now())
                    .and_then(|p| s.factions.get(&p.captor_faction)?.prison.clone())
            })
        };
        if let Some(loc) = prison {
            let (x, _, z) = event.to_position;
            if event.player.get_world().get_id() != loc.world
                || (x - loc.x).abs() > 10.0
                || (z - loc.z).abs() > 10.0
            {
                event.cancelled = true;
            }
        }
        event
    }
}
struct Break(Arc<App>);
impl EventHandler<BlockBreakEvent> for Break {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockBreakEvent>,
    ) -> EventData<BlockBreakEvent> {
        if let Some(p) = &event.player {
            let claim = Claim {
                world: p.get_world().get_id(),
                chunk_x: event.block_pos.x.div_euclid(16),
                chunk_z: event.block_pos.z.div_euclid(16),
            };
            if !self.0.can_build(p, &claim) {
                event.cancelled = true;
                p.send_system_message(
                    TextComponent::text("This territory is protected by another faction."),
                    false,
                )
            }
        }
        event
    }
}
struct Place(Arc<App>);
impl EventHandler<BlockPlaceEvent> for Place {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockPlaceEvent>,
    ) -> EventData<BlockPlaceEvent> {
        let p = &event.player;
        let claim = Claim {
            world: p.get_world().get_id(),
            chunk_x: event.block_pos.x.div_euclid(16),
            chunk_z: event.block_pos.z.div_euclid(16),
        };
        if !self.0.can_build(p, &claim) {
            event.cancelled = true;
            p.send_system_message(
                TextComponent::text("This territory is protected by another faction."),
                false,
            )
        }
        event
    }
}
struct Damage(Arc<App>);
impl EventHandler<EntityDamageByEntityEvent> for Damage {
    fn handle(
        &self,
        server: Server,
        mut event: EventData<EntityDamageByEntityEvent>,
    ) -> EventData<EntityDamageByEntityEvent> {
        let (Some(victim), Some(attacker)) = (
            App::player_by_entity(&server, event.entity_id),
            App::player_by_entity(&server, event.damager_id),
        ) else {
            return event;
        };
        let (v, a) = (pid(&victim), pid(&attacker));
        let cancel = {
            let s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            match (s.player_faction.get(&a), s.player_faction.get(&v)) {
                (Some(x), Some(y)) if x == y => true,
                (Some(x), Some(y)) if s.relation(x, y) == Relation::Ally => true,
                (Some(x), Some(y)) if s.relation(x, y) == Relation::Truce => true,
                _ => false,
            }
        };
        if cancel {
            event.cancelled = true;
            attacker.send_system_message(
                TextComponent::text("Friendly fire is disabled for faction allies and truces."),
                false,
            )
        } else {
            self.0
                .last_hits
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(event.entity_id, (a, App::now()));
        }
        event
    }
}
struct Death(Arc<App>);
impl EventHandler<PlayerDeathEvent> for Death {
    fn handle(
        &self,
        server: Server,
        event: EventData<PlayerDeathEvent>,
    ) -> EventData<PlayerDeathEvent> {
        let victim = pid(&event.player);
        let entity_id = event.player.as_entity().get_id() as i32;
        let killer = self
            .0
            .last_hits
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&entity_id)
            .filter(|(_, at)| App::now() - *at <= 30)
            .map(|v| v.0);
        let now = App::now();
        let _ = self.0.mutate(&victim, "death", |s| {
            let Some(vf) = s.player_faction.get(&victim).cloned() else {
                return Ok(());
            };
            if let Some(f) = s.factions.get_mut(&vf) {
                f.power = (f.power - self.0.config.factions.power_loss_on_death).max(0)
            }
            let Some(killer) = killer else { return Ok(()) };
            let Some(kf) = s.player_faction.get(&killer).cloned() else {
                return Ok(());
            };
            let Some(war_id) = s
                .wars
                .iter()
                .find(|(_, w)| {
                    w.status == WarStatus::Active
                        && ((w.attacker == kf && w.defender == vf)
                            || (w.attacker == vf && w.defender == kf))
                })
                .map(|(id, _)| id.clone())
            else {
                return Ok(());
            };
            let victim_is_leader = s.factions[&vf].leader == victim;
            let ransom = self.0.config.economy.pow_base_ransom
                + (s.factions[&vf].power - s.factions[&kf].power).abs() * 10;
            s.wars.get_mut(&war_id).unwrap().prisoners.insert(
                victim.clone(),
                Prisoner {
                    player: victim.clone(),
                    captor_faction: kf.clone(),
                    ransom,
                    release_at: now + self.0.config.war.prisoner_hours * 3600,
                },
            );
            if victim_is_leader {
                finish_war(s, &war_id, &kf, &vf, &self.0.config)
            }
            Ok(())
        });
        process_wars(&self.0, &server);
        event
    }
}
fn finish_war(s: &mut FactionState, id: &str, winner: &str, loser: &str, cfg: &config::Config) {
    let (power_diff, troops) = (
        s.factions[winner].power - s.factions[loser].power,
        s.factions[loser].members.len() as i64,
    );
    let amount = (cfg.economy.war_base_reparation + power_diff.abs() * 100 + troops * 50).max(0);
    let paid = amount.min(s.factions[loser].bank);
    s.factions.get_mut(loser).unwrap().bank -= paid;
    s.factions.get_mut(winner).unwrap().bank += paid;
    let w = s.wars.get_mut(id).unwrap();
    w.status = WarStatus::Finished;
    w.winner = Some(winner.into());
    w.loser = Some(loser.into());
    w.reparations = paid;
    if paid == amount {
        w.prisoners.clear();
    }
}
struct Click(Arc<App>);
impl EventHandler<InventoryClickEvent> for Click {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<InventoryClickEvent>,
    ) -> EventData<InventoryClickEvent> {
        let id = pid(&event.player);
        let view = self
            .0
            .menus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned();
        if view.as_deref() == Some("main") {
            event.cancelled = true;
            match event.raw_slot {
                10 => ui::open_mail(&self.0, &event.player),
                16 => {
                    let _ = ui::open_trade_inbox(&self.0, &event.player);
                }
                _ => {}
            }
        } else if view.as_deref() == Some("mail") {
            event.cancelled = true
        }
        event
    }
}
struct Close(Arc<App>);
impl EventHandler<InventoryCloseEvent> for Close {
    fn handle(
        &self,
        _: Server,
        event: EventData<InventoryCloseEvent>,
    ) -> EventData<InventoryCloseEvent> {
        let id = pid(&event.player);
        self.0
            .menus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        if let Some(view) = self
            .0
            .trades
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id)
        {
            let _ = self.0.mutate(&id, "trade-close", |s| {
                match view {
                    TradeView::Send { target, inventory } => {
                        let incoming = inventory
                            .get_all_items()
                            .into_iter()
                            .flatten()
                            .map(|i| TradeItem {
                                registry_key: i.get_registry_key(),
                                count: i.get_count(),
                            })
                            .collect::<Vec<_>>();
                        let box_items = s.trade.entry(target.clone()).or_default();
                        let remaining = self
                            .0
                            .config
                            .storage
                            .trade_slots
                            .saturating_sub(box_items.len());
                        box_items.extend(incoming.into_iter().take(remaining));
                    }
                    TradeView::Inbox { faction, inventory } => {
                        s.trade.insert(
                            faction,
                            inventory
                                .get_all_items()
                                .into_iter()
                                .flatten()
                                .map(|i| TradeItem {
                                    registry_key: i.get_registry_key(),
                                    count: i.get_count(),
                                })
                                .collect(),
                        );
                    }
                }
                Ok(())
            });
        }
        event
    }
}
struct Form(Arc<App>);
impl EventHandler<BedrockFormResponseEvent> for Form {
    fn handle(
        &self,
        _: Server,
        event: EventData<BedrockFormResponseEvent>,
    ) -> EventData<BedrockFormResponseEvent> {
        let view = self
            .0
            .forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&event.form_id);
        if view.as_deref() == Some("main")
            && let FormResponse::Simple(button) = FormResponse::parse(event.response_data.clone())
        {
            match button {
                0 => ui::open_mail(&self.0, &event.player),
                3 => {
                    let _ = ui::open_trade_inbox(&self.0, &event.player);
                }
                _ => {}
            }
        }
        event
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    #[test]
    fn permission_namespace_matches_plugin() {
        assert!(PERM_USER.starts_with("CalabazaFactions:"));
        assert!(PERM_ADMIN.starts_with("CalabazaFactions:"));
    }
    #[test]
    fn defaults_match_requested_war_times() {
        let c = config::Config::default();
        assert_eq!(c.war.request_hours, 72);
        assert_eq!(c.war.preparation_hours, 12);
        assert_eq!(c.war.battle_minutes, 30);
        assert_eq!(c.war.prisoner_hours, 24);
    }
    #[test]
    fn paid_war_settlement_releases_prisoners() {
        let mut s = FactionState::default();
        s.create("Attackers", "a", Visibility::Public, 1, 10)
            .unwrap();
        s.create("Defenders", "d", Visibility::Public, 1, 10)
            .unwrap();
        s.factions.get_mut("attackers").unwrap().bank = 10_000;
        s.wars.insert(
            "war".into(),
            War {
                id: "war".into(),
                attacker: "defenders".into(),
                defender: "attackers".into(),
                forced: false,
                status: WarStatus::Active,
                requested_at: 1,
                request_expires_at: 2,
                preparation_ends_at: None,
                battle_ends_at: Some(3),
                ready: Default::default(),
                prisoners: [(
                    "p".into(),
                    Prisoner {
                        player: "p".into(),
                        captor_faction: "defenders".into(),
                        ransom: 250,
                        release_at: 100,
                    },
                )]
                .into(),
                winner: None,
                loser: None,
                reparations: 0,
            },
        );
        finish_war(
            &mut s,
            "war",
            "defenders",
            "attackers",
            &config::Config::default(),
        );
        assert!(s.wars["war"].prisoners.is_empty());
        assert_eq!(s.wars["war"].status, WarStatus::Finished);
    }
}
