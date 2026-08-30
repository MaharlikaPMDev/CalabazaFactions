mod app;
mod config;
pub mod domain;
mod storage;
mod ui;

use app::{App, ArenaSetup, TradeView, ZoneSetup};
use config::RankPermission;
use domain::*;
use pumpkin_plugin_api::{
    Context, Plugin, PluginMetadata, Server,
    command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
        StringType,
    },
    commands::CommandHandler,
    common::{Hand, Locale},
    events::{
        BedrockFormResponseEvent, BlockBreakEvent, BlockExplodeEvent, BlockFromToEvent,
        BlockPistonExtendEvent, BlockPistonRetractEvent, BlockPlaceEvent, EntityChangeBlockEvent,
        EntityDamageByEntityEvent, EntityExplodeEvent, EventData, EventHandler, EventPriority,
        FluidLevelChangeEvent, InventoryClickEvent, InventoryCloseEvent, InventoryMoveItemEvent,
        PlayerBucketEmptyEvent, PlayerBucketFillEvent, PlayerDeathEvent, PlayerInteractEvent,
        PlayerJoinEvent, PlayerLeaveEvent, PlayerMoveEvent,
    },
    permission::{Permission, PermissionDefault, PermissionLevel},
    permissions,
    text::TextComponent,
};
use std::{collections::HashSet, path::PathBuf, sync::Arc};

const PERM_USER: &str = "CalabazaFactions:command.faction";
const PERM_ADMIN: &str = "CalabazaFactions:command.admin";
struct CalabazaFactions {
    app: Option<Arc<App>>,
}

impl Plugin for CalabazaFactions {
    fn new() -> Self {
        Self { app: None }
    }
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata{name:"CalabazaFactions".into(),version:env!("CARGO_PKG_VERSION").into(),authors:vec!["MaharlikaPMDev".into()],description:"Playable factions, claims, diplomacy, wars, POWs, economy, mail, and alliance trade for PumpkinMC.".into(),dependencies:vec![],permissions:vec![permissions::FS_READ_DATA.into(),permissions::FS_WRITE_DATA.into()]}
    }
    fn on_load(&mut self, context: Context) -> pumpkin_plugin_api::Result<()> {
        pumpkin_plugin_api::i18n::load_translations(
            "calabazafactions",
            include_str!("../i18n/en_us.json"),
            Locale::EnUs,
        );
        pumpkin_plugin_api::i18n::load_translations(
            "calabazafactions",
            include_str!("../i18n/fil_ph.json"),
            Locale::FilPh,
        );
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
        context.register_event_handler(Interact(app.clone()), EventPriority::Highest, true)?;
        context.register_event_handler(Click(app.clone()), EventPriority::Highest, true)?;
        context.register_event_handler(Close(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(Form(app.clone()), EventPriority::Normal, true)?;
        context.register_event_handler(PistonExtend(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(PistonRetract(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(BlockExplosion(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(EntityExplosion(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(FluidFlow(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(FluidLevel(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(EntityGrief(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(InventoryMove(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(BucketEmpty(app.clone()), EventPriority::High, true)?;
        context.register_event_handler(BucketFill(app.clone()), EventPriority::High, true)?;
        tracing::info!(
            "CalabazaFactions v{} loaded without a scheduler",
            env!("CARGO_PKG_VERSION")
        );
        self.app = Some(app);
        Ok(())
    }

    fn handle_ipc_message(&mut self, _sender: String, message: Vec<u8>) -> Result<Vec<u8>, String> {
        let app = self.app.as_ref().ok_or("plugin is not loaded")?;
        let request: serde_json::Value =
            serde_json::from_slice(&message).map_err(|e| format!("invalid request: {e}"))?;
        let action = request
            .get("action")
            .and_then(serde_json::Value::as_str)
            .ok_or("missing action")?;
        let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
        let response = match action {
            "faction" => {
                let player = request
                    .get("player")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("missing player")?;
                let faction = state.faction_of(player);
                serde_json::json!({
                    "version": 1,
                    "faction_id": state.faction_id(player),
                    "faction_name": faction.map(|f| f.name.as_str()),
                })
            }
            "relation" => {
                let first = request
                    .get("first")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("missing first")?;
                let second = request
                    .get("second")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("missing second")?;
                serde_json::json!({
                    "version": 1,
                    "relation": state.relation_between_players(first, second),
                })
            }
            _ => return Err("unsupported action".into()),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
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
        let result = execute(
            &self.app,
            &server,
            &player,
            &input,
            sender.has_permission(&server, PERM_ADMIN),
        );
        if result.is_ok() {
            ui::update_scoreboard(&self.app, &player);
        }
        match result {
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
fn require_permission(
    app: &App,
    s: &FactionState,
    p: &str,
    permission: RankPermission,
) -> Result<String, String> {
    let id = require_faction(s, p)?;
    if !app.rank_allows(s, p, permission) {
        return Err(format!(
            "your faction rank does not grant {permission:?} permission"
        ));
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

fn send_help(player: &pumpkin_plugin_api::Player, requested_page: &str) -> Result<(), String> {
    let page = requested_page
        .parse::<usize>()
        .map_err(|_| "usage: /faction help [1|2|3]")?;
    let lines: &[&str] = match page {
        1 => &[
            "§6§lCalabazaFactions Help §7• §fIdentity §8(1/3)",
            "§e/faction§7 — Open your faction page",
            "§e/faction create <name> [public|private]§7 — Create a faction",
            "§e/faction info [faction]§7 — View a faction profile",
            "§e/faction setinfo <description>§7 — Edit your description",
            "§e/faction invite|apply|join|accept ...§7 — Recruit members",
            "§e/faction leave|kick <player>§7 — Change membership",
            "§e/faction role|transfer ...§7 — Manage leadership and ranks",
            "§e/faction public|private§7 — Change join policy",
            "§e/faction disband§7 — Permanently disband your faction",
            "§8Use §f/faction help 2 §8for territory and economy.",
        ],
        2 => &[
            "§6§lCalabazaFactions Help §7• §aTerritory & Economy §8(2/3)",
            "§e/faction claim|unclaim|overclaim§7 — Manage the current chunk",
            "§e/faction map§7 — Show relations and zones around you",
            "§e/faction sethome|home§7 — Set or visit faction home",
            "§e/faction setcore|core§7 — Set or visit the faction core",
            "§e/faction upgrade <power|territory|vault|shield>§7 — Upgrade the core",
            "§e/faction setbanner§7 — Capture the held item as faction banner",
            "§e/faction bank [balance]§7 — Show faction and wallet balances",
            "§e/faction bank deposit|withdraw <amount>§7 — Move funds",
            "§8Use §f/faction help 3 §8for diplomacy, war, and administration.",
        ],
        3 => &[
            "§6§lCalabazaFactions Help §7• §cWar, Mail & Admin §8(3/3)",
            "§e/faction relation <faction> <relation>§7 — Set diplomacy",
            "§e/faction shield§7 — Activate your configurable war shield",
            "§e/faction war|forcewar <faction>§7 — Start a war lifecycle",
            "§e/faction waraccept|wardecline|ready§7 — Answer or ready a war",
            "§e/faction setprison|paypow <player>§7 — Manage POWs",
            "§e/faction mail§7 — Open faction mail",
            "§e/faction trade <faction>|tradeinbox§7 — Exchange full item stacks",
            "§c/faction setarena|addarenaspawn|delarena§7 — Administer arenas",
            "§c/faction setzone|delzone|zones§7 — Administer safe/war zones",
            "§8Use §f/faction help 1 §8to return to identity commands.",
        ],
        _ => return Err("help page must be 1, 2, or 3".into()),
    };
    for line in lines {
        player.send_system_message(TextComponent::from_legacy_string(line), false);
    }
    Ok(())
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
    match action.as_str() {
        "menu" | "page" => {
            ui::open_faction(app, player);
            Ok(String::new())
        }
        "help" => {
            send_help(player, words.get(1).copied().unwrap_or("1"))?;
            Ok(String::new())
        }
        "create" => {
            let name = *words
                .get(1)
                .ok_or("usage: /faction create <name> [public|private]")?;
            let vis = if words
                .get(2)
                .is_some_and(|v| v.eq_ignore_ascii_case("private"))
            {
                Visibility::Private
            } else {
                Visibility::Public
            };
            let id = app.mutate(&player_id, "create", |s| {
                s.create(
                    name,
                    &player_id,
                    vis,
                    now,
                    app.config.factions.starting_power,
                )
            })?;
            Ok(format!("Faction {id} created."))
        }
        "disband" => {
            app.mutate(&player_id, "disband", |s| {
                let id = require_leader(s, &player_id)?;
                s.delete(&id)
            })?;
            Ok("Faction disbanded.".into())
        }
        "invite" => {
            let name = *words.get(1).ok_or("usage: /faction invite <player>")?;
            let target = resolve_player(app, server, name)
                .ok_or("player must have joined this server before")?;
            app.mutate(&player_id, "invite", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Members)?;
                s.invite(&id, &target, now + 72 * 3600)?;
                s.send_mail(
                    &id,
                    "Invitation sent",
                    &format!(
                        "{} invited {name}",
                        s.player_names.get(&player_id).cloned().unwrap_or_default()
                    ),
                    now,
                );
                Ok(())
            })?;
            Ok(format!("Invited {name}."))
        }
        "apply" => {
            let faction =
                FactionState::normalize(words.get(1).ok_or("usage: /faction apply <faction>")?);
            app.mutate(&player_id, "apply", |s| {
                s.apply(&faction, &player_id, now)?;
                s.send_mail(
                    &faction,
                    "New application",
                    &format!(
                        "{} applied to join.",
                        s.player_names
                            .get(&player_id)
                            .cloned()
                            .unwrap_or(player_id.clone())
                    ),
                    now,
                );
                Ok(())
            })?;
            Ok("Application sent through Faction Mail.".into())
        }
        "join" => {
            let faction =
                FactionState::normalize(words.get(1).ok_or("usage: /faction join <faction>")?);
            app.mutate(&player_id, "join", |s| {
                s.join(
                    &faction,
                    &player_id,
                    now,
                    app.config.factions.max_members,
                    true,
                )
            })?;
            Ok(format!("Joined {faction}."))
        }
        "accept" => {
            let name = *words.get(1).ok_or("usage: /faction accept <player>")?;
            let target = resolve_player(app, server, name).ok_or("unknown player")?;
            app.mutate(&player_id, "accept", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Members)?;
                if !s
                    .applications
                    .iter()
                    .any(|a| a.faction == id && a.player == target)
                {
                    return Err("no application from that player".into());
                }
                s.join(&id, &target, now, app.config.factions.max_members, false)
            })?;
            Ok(format!("Accepted {name}."))
        }
        "leave" => {
            app.mutate(&player_id, "leave", |s| s.leave(&player_id))?;
            Ok("You left your faction.".into())
        }
        "kick" => {
            let name = *words.get(1).ok_or("usage: /faction kick <player>")?;
            let target = resolve_player(app, server, name).ok_or("unknown player")?;
            app.mutate(&player_id, "kick", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Members)?;
                if s.factions[&id].leader == target {
                    return Err("cannot kick the leader".into());
                }
                s.factions
                    .get_mut(&id)
                    .ok_or("faction not found")?
                    .members
                    .remove(&target)
                    .ok_or("player is not a member")?;
                s.player_faction.remove(&target);
                Ok(())
            })?;
            Ok(format!("Kicked {name}."))
        }
        "role" => {
            let name = *words
                .get(1)
                .ok_or("usage: /faction role <player> <officer|veteran|member|recruit>")?;
            let role = Role::parse(words.get(2).ok_or("missing role")?).ok_or("invalid role")?;
            if role == Role::Leader {
                return Err("use /faction transfer".into());
            }
            let target = resolve_player(app, server, name).ok_or("unknown player")?;
            app.mutate(&player_id, "role", |s| {
                let id = require_leader(s, &player_id)?;
                s.factions
                    .get_mut(&id)
                    .unwrap()
                    .members
                    .get_mut(&target)
                    .map(|r| *r = role)
                    .ok_or_else(|| "player is not a member".to_string())
            })?;
            Ok(format!("Updated {name}'s role."))
        }
        "transfer" => {
            let name = *words.get(1).ok_or("usage: /faction transfer <player>")?;
            let target = resolve_player(app, server, name).ok_or("unknown player")?;
            app.mutate(&player_id, "transfer", |s| {
                let id = require_leader(s, &player_id)?;
                let f = s.factions.get_mut(&id).unwrap();
                if !f.members.contains_key(&target) {
                    return Err("player is not a member".into());
                }
                f.members.insert(player_id.clone(), Role::Officer);
                f.members.insert(target.clone(), Role::Leader);
                f.leader = target.clone();
                Ok(())
            })?;
            Ok(format!("Leadership transferred to {name}."))
        }
        "public" | "private" => {
            let vis = if action == "public" {
                Visibility::Public
            } else {
                Visibility::Private
            };
            app.mutate(&player_id, "visibility", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Members)?;
                s.factions.get_mut(&id).unwrap().visibility = vis;
                Ok(())
            })?;
            Ok(format!("Faction is now {action}."))
        }
        "setinfo" => {
            let info = words
                .get(1..)
                .ok_or("usage: /faction setinfo <description>")?
                .join(" ");
            if info.is_empty() {
                return Err("description cannot be empty".into());
            } else if info.chars().count() > 160 {
                return Err("description cannot exceed 160 characters".into());
            }
            app.mutate(&player_id, "setinfo", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Members)?;
                s.factions.get_mut(&id).unwrap().description = info;
                Ok(())
            })?;
            Ok("Faction description updated.".into())
        }
        "info" => {
            let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
            let id = if let Some(name) = words.get(1) {
                FactionState::normalize(name)
            } else {
                require_faction(&s, &player_id)?
            };
            let f = s.factions.get(&id).ok_or("faction not found")?;
            Ok(format!(
                "{} • {:?}\n{}\nPower {}/{} • Bank {} • Members {} • Claims {}",
                f.name,
                f.visibility,
                if f.description.is_empty() {
                    "No description set."
                } else {
                    &f.description
                },
                f.power,
                f.max_power,
                f.bank,
                f.members.len(),
                f.claims.len()
            ))
        }
        "claim" | "overclaim" | "unclaim" => territory_command(app, player, &player_id, &action),
        "map" => map_command(app, player, &player_id),
        "sethome" | "setprison" => {
            let loc = App::location(player);
            app.mutate(&player_id, &action, |s| {
                let permission = if action == "sethome" {
                    RankPermission::Home
                } else {
                    RankPermission::War
                };
                let id = require_permission(app, s, &player_id, permission)?;
                let f = s.factions.get_mut(&id).unwrap();
                if action == "sethome" {
                    f.home = Some(loc)
                } else {
                    f.prison = Some(loc)
                }
                Ok(())
            })?;
            Ok(format!(
                "Faction {} set.",
                if action == "sethome" {
                    "home"
                } else {
                    "prison"
                }
            ))
        }
        "home" => {
            let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
            let id = require_faction(&s, &player_id)?;
            let loc = s.factions[&id]
                .home
                .clone()
                .ok_or("faction home is not set")?;
            drop(s);
            teleport(server, player, &loc)?;
            Ok("Teleported to faction home.".into())
        }
        "setcore" => set_core(app, &player_id, player),
        "core" => teleport_to_core(app, server, &player_id, player),
        "upgrade" => upgrade_core(
            app,
            &player_id,
            words
                .get(1)
                .ok_or("usage: /faction upgrade <power|territory|vault|shield>")?,
        ),
        "setbanner" => set_banner(app, &player_id, player),
        "bank" => bank_command(app, &player_id, &words),
        "relation" => {
            let target = FactionState::normalize(
                words
                    .get(1)
                    .ok_or("usage: /faction relation <faction> <neutral|truce|ally|enemy>")?,
            );
            let relation = Relation::parse(words.get(2).ok_or("missing relation")?)
                .ok_or("invalid relation")?;
            app.mutate(&player_id, "relation", |s| {
                let id = require_permission(app, s, &player_id, RankPermission::Diplomacy)?;
                s.set_relation(&id, &target, relation.clone())?;
                s.send_mail(
                    &target,
                    "Diplomacy updated",
                    &format!("{id} set relations to {relation:?}."),
                    now,
                );
                Ok(())
            })?;
            Ok(format!("Relation with {target}: {relation:?}."))
        }
        "shield" => activate_shield(app, &player_id, now),
        "setarena" | "addarenaspawn" | "delarena" | "arenas" => {
            arena_command(app, &player_id, &action, &words, is_admin)
        }
        "setzone" | "delzone" | "zones" => zone_command(app, &player_id, &action, &words, is_admin),
        "war" | "forcewar" => start_war(
            app,
            &player_id,
            words.get(1).ok_or("usage: /faction war <faction>")?,
            action == "forcewar",
            now,
        ),
        "waraccept" | "wardecline" => answer_war(app, &player_id, action == "waraccept", now),
        "ready" => ready_war(app, server, &player_id, now),
        "paypow" => pay_pow(
            app,
            &player_id,
            words.get(1).ok_or("usage: /faction paypow <player>")?,
            server,
            now,
        ),
        "mail" => {
            ui::open_mail(app, player);
            Ok(String::new())
        }
        "trade" => {
            let target = FactionState::normalize(
                words
                    .get(1)
                    .ok_or("usage: /faction trade <allied faction>")?,
            );
            ui::open_trade_send(app, player, &target)?;
            Ok(String::new())
        }
        "tradeinbox" => {
            ui::open_trade_inbox(app, player)?;
            Ok(String::new())
        }
        _ => Err("Unknown subcommand. Use /faction help".into()),
    }
}

fn territory_command(
    app: &App,
    player: &pumpkin_plugin_api::Player,
    player_id: &str,
    action: &str,
) -> Result<String, String> {
    let claim = App::claim_at(player);
    match action {
        "claim" => {
            app.mutate(player_id, "claim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                let level = state.factions[&id].upgrade_level(UpgradeKind::Territory);
                let bonus = usize::from(level) * app.config.cores.claims_per_level;
                state.claim_with_bonus(&id, claim, bonus)
            })?;
            Ok("Chunk claimed.".into())
        }
        "overclaim" => {
            let previous = app.mutate(player_id, "overclaim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                let level = state.factions[&id].upgrade_level(UpgradeKind::Territory);
                let bonus = usize::from(level) * app.config.cores.claims_per_level;
                state.overclaim_with_bonus(&id, claim, bonus)
            })?;
            Ok(format!("Overclaimed this chunk from {previous}."))
        }
        "unclaim" => {
            app.mutate(player_id, "unclaim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                if !state.factions.get_mut(&id).unwrap().claims.remove(&claim) {
                    return Err("your faction does not own this chunk".into());
                }
                Ok(())
            })?;
            Ok("Chunk unclaimed.".into())
        }
        _ => unreachable!(),
    }
}

fn map_command(
    app: &App,
    player: &pumpkin_plugin_api::Player,
    player_id: &str,
) -> Result<String, String> {
    let center = App::claim_at(player);
    let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let own = state.player_faction.get(player_id);
    let mut output = format!(
        "Territory map • {} ({}, {})\n",
        center.world, center.chunk_x, center.chunk_z
    );
    for dz in -4..=4 {
        for dx in -4..=4 {
            if dx == 0 && dz == 0 {
                output.push('@');
                continue;
            }
            let chunk = Claim {
                world: center.world.clone(),
                chunk_x: center.chunk_x + dx,
                chunk_z: center.chunk_z + dz,
            };
            let block_x = chunk.chunk_x * 16 + 8;
            let block_z = chunk.chunk_z * 16 + 8;
            let marker = if let Some(zone) = state.zone_at(&chunk.world, block_x, block_z) {
                match zone.kind {
                    ZoneKind::Safe => 'S',
                    ZoneKind::War => 'W',
                }
            } else if let Some(owner) = state.claim_owner(&chunk) {
                if own.is_some_and(|id| id == &owner.id) {
                    'O'
                } else if let Some(id) = own {
                    match state.relation(id, &owner.id) {
                        Relation::Ally => 'A',
                        Relation::Enemy => 'E',
                        Relation::Truce => 'T',
                        Relation::Neutral => '#',
                    }
                } else {
                    '#'
                }
            } else {
                '.'
            };
            output.push(marker);
        }
        output.push('\n');
    }
    output.push_str("@ you • O own • A ally • T truce • E enemy • S safe • W war • . wild");
    Ok(output)
}

fn set_core(
    app: &App,
    player_id: &str,
    player: &pumpkin_plugin_api::Player,
) -> Result<String, String> {
    let location = App::location(player);
    app.mutate(player_id, "set-core", |state| {
        let id = require_permission(app, state, player_id, RankPermission::Core)?;
        state.factions.get_mut(&id).unwrap().core = Some(location);
        Ok(())
    })?;
    Ok("Faction core set.".into())
}

fn teleport_to_core(
    app: &App,
    server: &Server,
    player_id: &str,
    player: &pumpkin_plugin_api::Player,
) -> Result<String, String> {
    let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let id = require_faction(&state, player_id)?;
    let location = state.factions[&id]
        .core
        .clone()
        .ok_or("faction core is not set")?;
    drop(state);
    teleport(server, player, &location)?;
    Ok("Teleported to the faction core.".into())
}

fn upgrade_core(app: &App, player_id: &str, value: &str) -> Result<String, String> {
    let kind = UpgradeKind::parse(value).ok_or("unknown upgrade")?;
    let (level, cost) = app.mutate(player_id, "core-upgrade", |state| {
        let id = require_permission(app, state, player_id, RankPermission::Core)?;
        let faction = state.factions.get_mut(&id).unwrap();
        if faction.core.is_none() {
            return Err("set a faction core before purchasing upgrades".into());
        }
        let current = faction.upgrade_level(kind.clone());
        if current >= app.config.cores.max_upgrade_level {
            return Err("that upgrade is already at maximum level".into());
        }
        let level = current + 1;
        let cost = app.config.cores.base_upgrade_cost * i64::from(level);
        if faction.bank < cost {
            return Err(format!("faction bank needs {cost} for this upgrade"));
        }
        faction.bank -= cost;
        faction.upgrades.insert(kind.clone(), level);
        Ok((level, cost))
    })?;
    Ok(format!("Upgraded {kind:?} to level {level} for {cost}."))
}

fn set_banner(
    app: &App,
    player_id: &str,
    player: &pumpkin_plugin_api::Player,
) -> Result<String, String> {
    let held = player
        .get_item_in_hand(Hand::Right)
        .ok_or("hold the banner item in your main hand")?;
    let banner = ui::serialize_item(&held);
    app.mutate(player_id, "set-banner", |state| {
        let id = require_permission(app, state, player_id, RankPermission::Core)?;
        state.factions.get_mut(&id).unwrap().banner = Some(banner);
        Ok(())
    })?;
    Ok("Faction banner captured with all item components.".into())
}

fn activate_shield(app: &App, player_id: &str, now: u64) -> Result<String, String> {
    let until = app.mutate(player_id, "war-shield", |state| {
        let id = require_permission(app, state, player_id, RankPermission::War)?;
        if state.wars.values().any(|war| {
            matches!(
                war.status,
                WarStatus::Requested | WarStatus::Preparing | WarStatus::Active
            ) && (war.attacker == id || war.defender == id)
        }) {
            return Err("a shield cannot be activated during a live war lifecycle".into());
        }
        let faction = state.factions.get_mut(&id).unwrap();
        if faction.war_policy.shield_until > now {
            return Err("your faction already has an active shield".into());
        }
        if faction.war_policy.cooldown_until > now || faction.war_policy.grace_until > now {
            return Err("your faction must finish its cooldown and grace period first".into());
        }
        let bonus = u64::from(faction.upgrade_level(UpgradeKind::Shield))
            * app.config.cores.shield_hours_per_level;
        let until = now + (app.config.war.shield_hours + bonus) * 3600;
        faction.war_policy.shield_until = until;
        faction.war_policy.cooldown_until = until + app.config.war.cooldown_hours * 3600;
        Ok(until)
    })?;
    Ok(format!("War shield active until Unix time {until}."))
}

fn arena_command(
    app: &App,
    player_id: &str,
    action: &str,
    words: &[&str],
    is_admin: bool,
) -> Result<String, String> {
    if !is_admin {
        return Err("admin permission required".into());
    }
    match action {
        "setarena" => {
            let arena = FactionState::normalize(words.get(1).copied().unwrap_or("default"));
            if arena.is_empty() {
                return Err("arena name must contain a letter or number".into());
            }
            app.arena_setup
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(
                    player_id.into(),
                    ArenaSetup::Pair {
                        arena: arena.clone(),
                        team1: None,
                    },
                );
            Ok(format!(
                "Arena '{arena}' setup started. Tap Team 1's first spawn block."
            ))
        }
        "addarenaspawn" => {
            let arena = FactionState::normalize(
                words
                    .get(1)
                    .ok_or("usage: /faction addarenaspawn <arena> <1|2>")?,
            );
            let side = words
                .get(2)
                .ok_or("missing side (1 or 2)")?
                .parse::<u8>()
                .map_err(|_| "side must be 1 or 2")?;
            if !matches!(side, 1 | 2) {
                return Err("side must be 1 or 2".into());
            }
            app.arena_setup
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(player_id.into(), ArenaSetup::Spawn { arena, side });
            Ok("Tap the block for the additional arena spawn.".into())
        }
        "delarena" => {
            let arena =
                FactionState::normalize(words.get(1).ok_or("usage: /faction delarena <arena>")?);
            app.mutate(player_id, "delete-arena", |state| {
                state.arenas.remove(&arena).ok_or("arena not found")?;
                Ok(())
            })?;
            Ok(format!("Arena '{arena}' deleted."))
        }
        "arenas" => {
            let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.arenas.is_empty() {
                return Ok("No arenas configured.".into());
            }
            Ok(state
                .arenas
                .values()
                .map(|arena| {
                    format!(
                        "{}: {} Team 1 / {} Team 2 spawn(s){}",
                        arena.id,
                        arena.team1_spawns.len(),
                        arena.team2_spawns.len(),
                        if arena.enabled { "" } else { " (disabled)" }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
        _ => unreachable!(),
    }
}

fn zone_command(
    app: &App,
    player_id: &str,
    action: &str,
    words: &[&str],
    is_admin: bool,
) -> Result<String, String> {
    if !is_admin {
        return Err("admin permission required".into());
    }
    match action {
        "setzone" => {
            let id = FactionState::normalize(
                words
                    .get(1)
                    .ok_or("usage: /faction setzone <name> <safe|war>")?,
            );
            let kind = ZoneKind::parse(words.get(2).ok_or("missing zone type")?)
                .ok_or("zone type must be safe or war")?;
            app.zone_setup
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(
                    player_id.into(),
                    ZoneSetup {
                        id: id.clone(),
                        kind,
                        first: None,
                    },
                );
            Ok(format!(
                "Zone '{id}' setup started. Tap the first corner block."
            ))
        }
        "delzone" => {
            let id = FactionState::normalize(words.get(1).ok_or("usage: /faction delzone <name>")?);
            app.mutate(player_id, "delete-zone", |state| {
                state.zones.remove(&id).ok_or("zone not found")?;
                Ok(())
            })?;
            Ok(format!("Zone '{id}' deleted."))
        }
        "zones" => {
            let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.zones.is_empty() {
                return Ok("No safe or war zones configured.".into());
            }
            Ok(state
                .zones
                .values()
                .map(|zone| {
                    format!(
                        "{}: {:?} {} ({}, {}) to ({}, {})",
                        zone.id,
                        zone.kind,
                        zone.world,
                        zone.min_x,
                        zone.min_z,
                        zone.max_x,
                        zone.max_z
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
        _ => unreachable!(),
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
                if !app.rank_allows(s, p, RankPermission::Economy) {
                    return Err("your faction rank cannot withdraw from the bank".into());
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
        let attacker = require_permission(app, s, p, RankPermission::War)?;
        if s.war_slot_busy() {
            return Err(
                "the global war slot is occupied; wait for the current request or war to finish"
                    .into(),
            );
        }
        if !s.factions.contains_key(&target) {
            return Err("target faction not found".into());
        }
        if attacker == target {
            return Err("a faction cannot declare war on itself".into());
        }
        if let Some(reason) = s.war_block_reason(&attacker, &target, now) {
            return Err(reason);
        }
        if s.factions[&attacker].prison.is_none() || s.factions[&target].prison.is_none() {
            return Err("both factions must set a prison before war".into());
        }
        if !forced && s.relation(&attacker, &target) != Relation::Enemy {
            return Err("declare the target an enemy first".into());
        }
        let arena_id = s.select_arena()?;
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
                arena_id,
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
        let faction = require_permission(app, s, p, RankPermission::War)?;
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
        let faction = require_permission(app, s, p, RankPermission::War)?;
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
        let faction = require_permission(app, s, p, RankPermission::War)?;
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
    let started = {
        let mut state = app.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut candidate = state.clone();
        let mut changed = false;
        let mut started = false;
        let timed_out = candidate
            .wars
            .values()
            .filter(|w| w.status == WarStatus::Active && w.battle_ends_at.is_some_and(|v| v <= now))
            .map(|w| (w.id.clone(), w.defender.clone(), w.attacker.clone()))
            .collect::<Vec<_>>();
        for w in candidate.wars.values_mut() {
            match w.status {
                WarStatus::Requested if w.request_expires_at <= now => {
                    w.status = WarStatus::Expired;
                    changed = true
                }
                WarStatus::Preparing if w.preparation_ends_at.is_some_and(|v| v <= now) => {
                    w.status = WarStatus::Active;
                    w.battle_ends_at = Some(now + app.config.war.battle_minutes * 60);
                    changed = true;
                    started = true;
                }
                _ => {}
            }
            let before = w.prisoners.len();
            w.prisoners.retain(|_, prisoner| prisoner.release_at > now);
            changed |= before != w.prisoners.len();
        }
        for (id, winner, loser) in timed_out {
            finish_war(&mut candidate, &id, &winner, &loser, &app.config);
            changed = true;
        }
        if changed {
            match storage::save(&app.data_dir, &candidate) {
                Ok(()) => *state = candidate,
                Err(error) => {
                    tracing::error!("failed to persist timed war transition: {error}");
                    started = false;
                }
            }
        }
        started
    };
    if started {
        teleport_active_wars(app, server)
    }
}
fn teleport_active_wars(app: &App, server: &Server) {
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let Some(war) = s
        .wars
        .values()
        .filter(|w| w.status == WarStatus::Active)
        .min_by_key(|w| w.requested_at)
    else {
        return;
    };
    let Some(arena) = s.arenas.get(&war.arena_id).cloned() else {
        return;
    };
    let team1 = s.factions[&war.attacker]
        .members
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    let team2 = s.factions[&war.defender]
        .members
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    drop(s);
    let mut team1_index = 0usize;
    let mut team2_index = 0usize;
    for p in server.get_all_players() {
        let id = pid(&p);
        let spawn = if team1.contains(&id) {
            let spawn = arena
                .team1_spawns
                .get(team1_index % arena.team1_spawns.len());
            team1_index += 1;
            spawn
        } else if team2.contains(&id) {
            let spawn = arena
                .team2_spawns
                .get(team2_index % arena.team2_spawns.len());
            team2_index += 1;
            spawn
        } else {
            None
        };
        if let Some(spawn) = spawn {
            let _ = teleport(server, &p, spawn);
            p.send_system_message(TextComponent::text(&format!("The faction war has begun in arena '{}'. Attackers have {} minutes to defeat the defending leader.", arena.id, app.config.war.battle_minutes)),false);
        }
    }
}

struct Interact(Arc<App>);
impl EventHandler<PlayerInteractEvent> for Interact {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<PlayerInteractEvent>,
    ) -> EventData<PlayerInteractEvent> {
        let id = pid(&event.player);
        let arena_setup = self
            .0
            .arena_setup
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned();
        if let (Some(setup), Some(pos)) = (arena_setup, event.clicked_pos) {
            event.cancelled = true;
            let location = Location {
                world: event.player.get_world().get_id(),
                x: f64::from(pos.x) + 0.5,
                y: f64::from(pos.y) + 1.0,
                z: f64::from(pos.z) + 0.5,
            };
            match setup {
                ArenaSetup::Pair { arena, team1: None } => {
                    self.0
                        .arena_setup
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .insert(
                            id,
                            ArenaSetup::Pair {
                                arena,
                                team1: Some(location),
                            },
                        );
                    event.player.send_system_message(
                        TextComponent::text(
                            "Team 1 spawn recorded. Tap Team 2's first spawn block.",
                        ),
                        false,
                    );
                }
                ArenaSetup::Pair {
                    arena,
                    team1: Some(team1),
                } => {
                    self.0
                        .arena_setup
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(&id);
                    let result = self.0.mutate(&id, "set-arena", |state| {
                        state.arenas.insert(
                            arena.clone(),
                            Arena {
                                id: arena.clone(),
                                team1_spawns: vec![team1],
                                team2_spawns: vec![location],
                                enabled: true,
                            },
                        );
                        Ok(())
                    });
                    let message = result
                        .map(|()| format!("Arena '{arena}' saved with both spawn groups."))
                        .unwrap_or_else(|error| error);
                    event
                        .player
                        .send_system_message(TextComponent::text(&message), false);
                }
                ArenaSetup::Spawn { arena, side } => {
                    self.0
                        .arena_setup
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(&id);
                    let result = self.0.mutate(&id, "add-arena-spawn", |state| {
                        let entry = state.arenas.entry(arena.clone()).or_insert_with(|| Arena {
                            id: arena.clone(),
                            team1_spawns: vec![],
                            team2_spawns: vec![],
                            enabled: true,
                        });
                        if side == 1 {
                            entry.team1_spawns.push(location);
                        } else {
                            entry.team2_spawns.push(location);
                        }
                        Ok(())
                    });
                    let message = result
                        .map(|()| format!("Added a Team {side} spawn to arena '{arena}'."))
                        .unwrap_or_else(|error| error);
                    event
                        .player
                        .send_system_message(TextComponent::text(&message), false);
                }
            }
            return event;
        }

        let zone_setup = self
            .0
            .zone_setup
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned();
        if let (Some(setup), Some(pos)) = (zone_setup, event.clicked_pos) {
            event.cancelled = true;
            let location = Location {
                world: event.player.get_world().get_id(),
                x: f64::from(pos.x),
                y: f64::from(pos.y),
                z: f64::from(pos.z),
            };
            if let Some(first) = setup.first {
                self.0
                    .zone_setup
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&id);
                let result = self.0.mutate(&id, "set-zone", |state| {
                    state.set_zone(&setup.id, setup.kind.clone(), &first, &location)
                });
                let message = result
                    .map(|()| format!("Zone '{}' saved.", setup.id))
                    .unwrap_or_else(|error| error);
                event
                    .player
                    .send_system_message(TextComponent::text(&message), false);
            } else {
                self.0
                    .zone_setup
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(
                        id,
                        ZoneSetup {
                            first: Some(location),
                            ..setup
                        },
                    );
                event.player.send_system_message(
                    TextComponent::text("First corner recorded. Tap the opposite corner."),
                    false,
                );
            }
            return event;
        }

        if self.0.config.protection.containers
            && is_container_block(&event.block)
            && let Some(pos) = event.clicked_pos
            && !self.0.can_build_at(&event.player, pos.x, pos.z)
        {
            event.cancelled = true;
            event
                .player
                .send_system_message(TextComponent::text("That container is protected."), false);
        }
        event
    }
}

fn is_container_block(block: &str) -> bool {
    [
        "chest",
        "barrel",
        "shulker_box",
        "hopper",
        "furnace",
        "smoker",
        "blast_furnace",
        "dispenser",
        "dropper",
        "brewing_stand",
        "crafter",
        "decorated_pot",
    ]
    .iter()
    .any(|name| block.contains(name))
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
        ui::update_scoreboard(&self.0, &event.player);
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
        self.0
            .arena_setup
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        self.0
            .zone_setup
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        self.0
            .trades
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id);
        self.0
            .scoreboards
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
        if let Some(p) = &event.player
            && !self.0.can_build_at(p, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
            p.send_system_message(
                TextComponent::text("This territory is protected by another faction."),
                false,
            )
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
        if !self.0.can_build_at(p, event.block_pos.x, event.block_pos.z) {
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

fn environment_is_protected(app: &App, world: Option<&str>, x: i32, z: i32) -> bool {
    app.state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .environmental_protected(world, x, z)
}

struct PistonExtend(Arc<App>);
impl EventHandler<BlockPistonExtendEvent> for PistonExtend {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockPistonExtendEvent>,
    ) -> EventData<BlockPistonExtendEvent> {
        if self.0.config.protection.pistons
            && environment_is_protected(&self.0, None, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct PistonRetract(Arc<App>);
impl EventHandler<BlockPistonRetractEvent> for PistonRetract {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockPistonRetractEvent>,
    ) -> EventData<BlockPistonRetractEvent> {
        if self.0.config.protection.pistons
            && environment_is_protected(&self.0, None, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct BlockExplosion(Arc<App>);
impl EventHandler<BlockExplodeEvent> for BlockExplosion {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockExplodeEvent>,
    ) -> EventData<BlockExplodeEvent> {
        if self.0.config.protection.explosions
            && environment_is_protected(&self.0, None, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct EntityExplosion(Arc<App>);
impl EventHandler<EntityExplodeEvent> for EntityExplosion {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<EntityExplodeEvent>,
    ) -> EventData<EntityExplodeEvent> {
        let (x, _, z) = event.position;
        if self.0.config.protection.explosions
            && environment_is_protected(&self.0, None, x.floor() as i32, z.floor() as i32)
        {
            event.cancelled = true;
        }
        event
    }
}

struct FluidFlow(Arc<App>);
impl EventHandler<BlockFromToEvent> for FluidFlow {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<BlockFromToEvent>,
    ) -> EventData<BlockFromToEvent> {
        if self.0.config.protection.fluids
            && environment_is_protected(&self.0, None, event.to_pos.x, event.to_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct FluidLevel(Arc<App>);
impl EventHandler<FluidLevelChangeEvent> for FluidLevel {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<FluidLevelChangeEvent>,
    ) -> EventData<FluidLevelChangeEvent> {
        let world = event.target_world.get_id();
        if self.0.config.protection.fluids
            && environment_is_protected(&self.0, Some(&world), event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct EntityGrief(Arc<App>);
impl EventHandler<EntityChangeBlockEvent> for EntityGrief {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<EntityChangeBlockEvent>,
    ) -> EventData<EntityChangeBlockEvent> {
        if self.0.config.protection.entity_grief
            && environment_is_protected(&self.0, None, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct InventoryMove(Arc<App>);
impl EventHandler<InventoryMoveItemEvent> for InventoryMove {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<InventoryMoveItemEvent>,
    ) -> EventData<InventoryMoveItemEvent> {
        if self.0.config.protection.containers
            && (environment_is_protected(&self.0, None, event.source_pos.x, event.source_pos.z)
                || environment_is_protected(&self.0, None, event.target_pos.x, event.target_pos.z))
        {
            event.cancelled = true;
        }
        event
    }
}

struct BucketEmpty(Arc<App>);
impl EventHandler<PlayerBucketEmptyEvent> for BucketEmpty {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<PlayerBucketEmptyEvent>,
    ) -> EventData<PlayerBucketEmptyEvent> {
        if self.0.config.protection.fluids
            && !self
                .0
                .can_build_at(&event.player, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

struct BucketFill(Arc<App>);
impl EventHandler<PlayerBucketFillEvent> for BucketFill {
    fn handle(
        &self,
        _: Server,
        mut event: EventData<PlayerBucketFillEvent>,
    ) -> EventData<PlayerBucketFillEvent> {
        if self.0.config.protection.fluids
            && !self
                .0
                .can_build_at(&event.player, event.block_pos.x, event.block_pos.z)
        {
            event.cancelled = true;
        }
        event
    }
}

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
        let a = pid(&attacker);
        let cancel = !self.0.can_pvp_at(&attacker, &victim);
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
    let raw_amount =
        (cfg.economy.war_base_reparation + power_diff.abs() * 100 + troops * 50).max(0);
    let vault_reduction =
        (i64::from(s.factions[loser].upgrade_level(UpgradeKind::Vault)) * 5).min(50);
    let amount = raw_amount * (100 - vault_reduction) / 100;
    let paid = amount.min(s.factions[loser].bank);
    s.factions.get_mut(loser).unwrap().bank -= paid;
    s.factions.get_mut(winner).unwrap().bank += paid;
    let now = App::now();
    for faction_id in [winner, loser] {
        let policy = &mut s.factions.get_mut(faction_id).unwrap().war_policy;
        policy.shield_until = 0;
        policy.cooldown_until = now + cfg.war.cooldown_hours * 3600;
        policy.grace_until = now + cfg.war.post_war_grace_hours * 3600;
    }
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
        // GUI host calls from inside InventoryClickEvent can re-enter Pumpkin's runtime and
        // freeze the server. Menus are deliberately informational; actions remain commands.
        event.cancelled = matches!(view.as_deref(), Some("main") | Some("mail"));
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
                            .map(|item| ui::serialize_item(&item))
                            .collect::<Vec<_>>();
                        let capacity = s
                            .factions
                            .get(&target)
                            .map(|faction| self.0.trade_capacity(faction))
                            .unwrap_or(self.0.config.storage.trade_slots);
                        let box_items = s.trade.entry(target.clone()).or_default();
                        let remaining = capacity.saturating_sub(box_items.len());
                        box_items.extend(incoming.into_iter().take(remaining));
                    }
                    TradeView::Inbox { faction, inventory } => {
                        s.trade.insert(
                            faction,
                            inventory
                                .get_all_items()
                                .into_iter()
                                .flatten()
                                .map(|item| ui::serialize_item(&item))
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
        self.0
            .forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&event.form_id);
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
                arena_id: "default".into(),
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
