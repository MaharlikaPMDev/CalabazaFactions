mod app;
mod config;
pub mod domain;
mod economy;
mod storage;
mod ui;

use app::{
    App, ArenaSetup, PendingTerritoryAction, TerritoryFormAction, TerritoryView, TradeView,
    ZoneSetup,
};
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
    scheduler::SchedulerExt,
    text::TextComponent,
    world::{BlockFlags, BlockPos},
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
        PluginMetadata{name:"CalabazaFactions".into(),version:env!("CARGO_PKG_VERSION").into(),authors:vec!["MaharlikaPMDev".into()],description:"Physical-core factions, strategic chunk territory, diplomacy, wars, economy, mail, and alliance trade for PumpkinMC.".into(),dependencies:vec![],permissions:vec![permissions::FS_READ_DATA.into(),permissions::FS_WRITE_DATA.into()]}
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

        let reconcile_app = app.clone();
        context.schedule_repeating_task(
            app.config.cores.reconcile_interval_ticks.max(1),
            app.config.cores.reconcile_interval_ticks.max(1),
            move |server| reconcile_cores(&reconcile_app, &server),
        );
        let delivery_app = app.clone();
        context.schedule_repeating_task(
            app.config.ipc.delivery_interval_ticks.max(1),
            app.config.ipc.delivery_interval_ticks.max(1),
            move |_server| deliver_ipc_events(&delivery_app),
        );
        if config::EconomyMode::External == app.config.economy.mode
            && let Err(error) = economy::health(&app.config.economy)
        {
            tracing::warn!("External economy is configured but not ready: {error}");
        }
        tracing::info!(
            "CalabazaFactions v{} loaded with core reconciliation and IPC delivery",
            env!("CARGO_PKG_VERSION")
        );
        self.app = Some(app);
        Ok(())
    }

    fn handle_ipc_message(&mut self, sender: String, message: Vec<u8>) -> Result<Vec<u8>, String> {
        let app = self.app.as_ref().ok_or("plugin is not loaded")?;
        let request: serde_json::Value =
            serde_json::from_slice(&message).map_err(|e| format!("invalid request: {e}"))?;
        if let Some(version) = request.get("version").and_then(serde_json::Value::as_u64)
            && version != 1
        {
            return Err(format!(
                "unsupported IPC schema version {version}; supported version is 1"
            ));
        }
        let action = request
            .get("action")
            .and_then(serde_json::Value::as_str)
            .ok_or("missing action")?;
        let response = match action {
            "faction" => {
                let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
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
                let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
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
            "capabilities" => serde_json::json!({
                "schema": "calabazafactions.ipc",
                "version": 1,
                "actions": ["capabilities", "faction", "relation", "subscribe", "unsubscribe", "events_since"],
                "event_schema": "calabazafactions.event",
                "topics": ["faction.*", "territory.*", "core.*", "war.*", "raid.*", "*"]
            }),
            "subscribe" => {
                let topics = ipc_topics(&request)?;
                app.ipc_subscribers
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(sender.clone(), topics.clone());
                serde_json::json!({
                    "schema": "calabazafactions.ipc",
                    "version": 1,
                    "subscriber": sender,
                    "topics": topics,
                })
            }
            "unsubscribe" => {
                let removed = app
                    .ipc_subscribers
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .remove(&sender)
                    .is_some();
                serde_json::json!({"schema": "calabazafactions.ipc", "version": 1, "removed": removed})
            }
            "events_since" => {
                let since = request
                    .get("since")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let topics = ipc_topics_optional(&request)?;
                let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
                let oldest = state
                    .events
                    .first()
                    .map_or(state.next_event_sequence, |event| event.sequence);
                let latest = state.events.last().map_or(0, |event| event.sequence);
                let events = state
                    .events
                    .iter()
                    .filter(|event| {
                        event.sequence > since && topic_matches(&topics, &event.event_type)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "schema": "calabazafactions.ipc",
                    "version": 1,
                    "oldest_sequence": oldest,
                    "latest_sequence": latest,
                    "gap": since > 0 && since.saturating_add(1) < oldest,
                    "events": events,
                })
            }
            _ => return Err("unsupported action".into()),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
    }
}
pumpkin_plugin_api::register_plugin!(CalabazaFactions);

fn ipc_topics(request: &serde_json::Value) -> Result<HashSet<String>, String> {
    let topics = request
        .get("topics")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing topics")?;
    let parsed = topics
        .iter()
        .map(|topic| {
            topic
                .as_str()
                .map(str::to_owned)
                .ok_or("topics must be strings")
        })
        .collect::<Result<HashSet<_>, _>>()?;
    if parsed.is_empty() {
        return Err("at least one topic is required".into());
    }
    const EXACT_TOPICS: &[&str] = &[
        "faction.created",
        "faction.disbanded",
        "core.established",
        "core.attacked",
        "core.destroyed",
        "core.restored",
        "territory.claimed",
        "territory.overclaimed",
        "territory.unclaimed",
        "war.declared",
        "war.accepted",
        "war.declined",
        "war.started",
        "war.ended",
        "raid.started",
        "raid.ended",
    ];
    const FAMILY_TOPICS: &[&str] = &["faction.*", "core.*", "territory.*", "war.*", "raid.*", "*"];
    if let Some(unknown) = parsed.iter().find(|topic| {
        !EXACT_TOPICS.contains(&topic.as_str()) && !FAMILY_TOPICS.contains(&topic.as_str())
    }) {
        return Err(format!("unknown IPC topic '{unknown}'"));
    }
    Ok(parsed)
}

fn ipc_topics_optional(request: &serde_json::Value) -> Result<HashSet<String>, String> {
    if request.get("topics").is_none() {
        return Ok(HashSet::from(["*".into()]));
    }
    ipc_topics(request)
}

fn topic_matches(topics: &HashSet<String>, event_type: &str) -> bool {
    topics.iter().any(|topic| {
        topic == "*"
            || topic == event_type
            || topic
                .strip_suffix(".*")
                .is_some_and(|prefix| event_type.starts_with(&format!("{prefix}.")))
    })
}

fn deliver_ipc_events(app: &App) {
    let subscribers = app
        .ipc_subscribers
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    let mut delivered = app
        .delivered_event_sequence
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let events = app
        .state
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .events
        .iter()
        .filter(|event| event.sequence > *delivered)
        .cloned()
        .collect::<Vec<_>>();
    for event in &events {
        let payload = match serde_json::to_vec(event) {
            Ok(payload) => payload,
            Err(error) => {
                tracing::warn!("Could not serialize IPC event {}: {error}", event.sequence);
                continue;
            }
        };
        for (plugin, topics) in &subscribers {
            if topic_matches(topics, &event.event_type)
                && let Err(error) = pumpkin_plugin_api::ipc::send_ipc_message(plugin, &payload)
            {
                tracing::warn!(
                    "IPC event {} delivery to {plugin} failed: {error:?}",
                    event.sequence
                );
            }
        }
        *delivered = event.sequence;
    }
}

fn reconcile_cores(app: &App, server: &Server) {
    let cores = {
        let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut cores = state
            .factions
            .iter()
            .filter_map(|(id, faction)| {
                faction
                    .has_active_core()
                    .then(|| faction.physical_core.clone())
                    .flatten()
                    .map(|core| (id.clone(), core))
            })
            .collect::<Vec<_>>();
        cores.sort_by(|left, right| left.0.cmp(&right.0));
        cores
    };
    if cores.is_empty() {
        return;
    }
    let mut cursor = app
        .reconcile_cursor
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let batch = app
        .config
        .cores
        .reconcile_batch_size
        .max(1)
        .min(cores.len());
    for offset in 0..batch {
        let (faction_id, core) = &cores[(*cursor + offset) % cores.len()];
        let Some(world) = server.get_world_by_name(&core.location.world) else {
            continue;
        };
        let claim = core.location.claim();
        if world.get_chunk(claim.chunk_x, claim.chunk_z).is_none() {
            continue;
        }
        let position = BlockPos {
            x: core.location.x,
            y: core.location.y,
            z: core.location.z,
        };
        if world.get_block(position).name == "minecraft:beacon" {
            continue;
        }
        let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
        if !world.set_block_by_name(position, "minecraft:beacon", flags) {
            tracing::warn!("Could not restore core for {faction_id}");
            continue;
        }
        let now = App::now();
        let _ = app.mutate("scheduler", "restore-core", |state| {
            state.push_event(
                "core.restored",
                now,
                serde_json::json!({"faction_id": faction_id, "location": core.location}),
            );
            Ok(())
        });
    }
    *cursor = (*cursor + batch) % cores.len();
}

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
            "§e/faction map§7 — Open the read-only territory map",
            "§e/faction territory§7 — Open strategic chunk management",
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
                let id = s.create(
                    name,
                    &player_id,
                    vis,
                    now,
                    app.config.factions.starting_power,
                )?;
                s.push_event(
                    "faction.created",
                    now,
                    serde_json::json!({"faction_id": id, "name": name, "leader": player_id}),
                );
                Ok(id)
            })?;
            Ok(format!("Faction {id} created."))
        }
        "disband" => {
            let active_core = {
                let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
                let id = require_leader(&state, &player_id)?;
                state.factions[&id]
                    .physical_core
                    .as_ref()
                    .map(|core| core.location.clone())
            };
            if let Some(location) = &active_core {
                let world = server
                    .get_world_by_name(&location.world)
                    .ok_or("the core world must be available before disbanding")?;
                if world
                    .get_chunk(location.x.div_euclid(16), location.z.div_euclid(16))
                    .is_none()
                {
                    return Err("the core chunk must be loaded before disbanding".into());
                }
            }
            let core_location = app.mutate(&player_id, "disband", |s| {
                let id = require_leader(s, &player_id)?;
                let name = s.factions[&id].name.clone();
                let core_location = s.factions[&id]
                    .physical_core
                    .as_ref()
                    .map(|core| core.location.clone());
                s.delete(&id)?;
                s.push_event(
                    "faction.disbanded",
                    now,
                    serde_json::json!({"faction_id": id, "name": name, "leader": player_id}),
                );
                Ok(core_location)
            })?;
            if let Some(location) = core_location
                && let Some(world) = server.get_world_by_name(&location.world)
                && world
                    .get_chunk(location.x.div_euclid(16), location.z.div_euclid(16))
                    .is_some()
            {
                let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
                let _ = world.set_block_by_name(
                    BlockPos {
                        x: location.x,
                        y: location.y,
                        z: location.z,
                    },
                    "minecraft:air",
                    flags,
                );
            }
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
        "claim" | "overclaim" | "unclaim" => {
            territory_command(app, server, player, &player_id, &action)
        }
        "territoryconfirm" => {
            let pending = app
                .pending_territory
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(&player_id)
                .ok_or("no territory action is awaiting confirmation")?;
            if pending.expires_at < now {
                return Err("the territory confirmation expired".into());
            }
            territory_at(app, server, &player_id, &pending.action, pending.claim)
        }
        "territorycancel" => {
            app.pending_territory
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(&player_id)
                .ok_or("no territory action is awaiting confirmation")?;
            Ok("Territory action cancelled.".into())
        }
        "map" | "territory" => {
            let management = action == "territory";
            if management {
                let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
                require_permission(app, &state, &player_id, RankPermission::Territory)?;
            }
            ui::open_territory(
                app,
                player,
                TerritoryView {
                    origin: App::claim_at(player),
                    offset_x: 0,
                    offset_z: 0,
                    management,
                    last_refresh_at: now
                        .saturating_sub(app.config.territory_ui.refresh_cooldown_seconds),
                },
            )?;
            Ok(String::new())
        }
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
        "setzone" | "convertzone" | "delzone" | "zones" | "zoneconfirm" | "zonecancel" => {
            zone_command(app, &player_id, &action, &words, is_admin)
        }
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
    server: &Server,
    player: &pumpkin_plugin_api::Player,
    player_id: &str,
    action: &str,
) -> Result<String, String> {
    territory_at(app, server, player_id, action, App::claim_at(player))
}

fn territory_at(
    app: &App,
    server: &Server,
    player_id: &str,
    action: &str,
    claim: Claim,
) -> Result<String, String> {
    let world = server
        .get_world_by_name(&claim.world)
        .ok_or("the selected world is unavailable")?;
    if world.get_chunk(claim.chunk_x, claim.chunk_z).is_none() {
        return Err("the selected chunk is unloaded; enter the area before changing it".into());
    }
    let border = world.get_border();
    let min_x = f64::from(claim.chunk_x * 16);
    let min_z = f64::from(claim.chunk_z * 16);
    if !border.contains(min_x, min_z) || !border.contains(min_x + 15.0, min_z + 15.0) {
        return Err("the complete chunk must be inside the world border".into());
    }
    match action {
        "claim" => {
            app.mutate(player_id, "claim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                if state.zone_at(
                    &claim.world,
                    claim.chunk_x * 16 + 8,
                    claim.chunk_z * 16 + 8,
                ).is_some() {
                    return Err("server-owned zones cannot be claimed".into());
                }
                if app.config.cores.max_claim_distance_from_core > 0 {
                    let core_claim = state.factions[&id]
                        .physical_core
                        .as_ref()
                        .ok_or("the faction core is not active")?
                        .location
                        .claim();
                    let distance = (core_claim.chunk_x - claim.chunk_x).abs()
                        + (core_claim.chunk_z - claim.chunk_z).abs();
                    if distance > app.config.cores.max_claim_distance_from_core {
                        return Err("the configured anti-corridor distance limit was reached".into());
                    }
                }
                if state.factions.iter().any(|(other_id, other)| {
                    other_id != &id
                        && other.physical_core.as_ref().is_some_and(|core| {
                            let core_claim = core.location.claim();
                            core_claim.world == claim.world
                                && (core_claim.chunk_x - claim.chunk_x).abs()
                                    <= app.config.cores.enemy_core_distance_chunks
                                && (core_claim.chunk_z - claim.chunk_z).abs()
                                    <= app.config.cores.enemy_core_distance_chunks
                        })
                }) {
                    return Err("the chunk is too close to another faction core".into());
                }
                let capacity = app.core_claim_capacity(&state.factions[&id]);
                state.strategic_claim(&id, claim.clone(), capacity)?;
                state.push_event(
                    "territory.claimed",
                    App::now(),
                    serde_json::json!({"faction_id": id, "world": claim.world, "chunk_x": claim.chunk_x, "chunk_z": claim.chunk_z}),
                );
                Ok(())
            })?;
            Ok("Chunk claimed.".into())
        }
        "overclaim" => {
            let previous = app.mutate(player_id, "overclaim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                if app.config.cores.max_claim_distance_from_core > 0 {
                    let core_claim = state.factions[&id]
                        .physical_core
                        .as_ref()
                        .ok_or("the faction core is not active")?
                        .location
                        .claim();
                    let distance = (core_claim.chunk_x - claim.chunk_x).abs()
                        + (core_claim.chunk_z - claim.chunk_z).abs();
                    if distance > app.config.cores.max_claim_distance_from_core {
                        return Err("the configured anti-corridor distance limit was reached".into());
                    }
                }
                if state.factions.iter().any(|(other_id, other)| {
                    other_id != &id
                        && other.physical_core.as_ref().is_some_and(|core| {
                            let core_claim = core.location.claim();
                            core_claim.world == claim.world
                                && (core_claim.chunk_x - claim.chunk_x).abs()
                                    <= app.config.cores.enemy_core_distance_chunks
                                && (core_claim.chunk_z - claim.chunk_z).abs()
                                    <= app.config.cores.enemy_core_distance_chunks
                        })
                }) {
                    return Err("the chunk is too close to another faction core".into());
                }
                let capacity = app.core_claim_capacity(&state.factions[&id]);
                let previous = state.strategic_overclaim(&id, claim.clone(), capacity)?;
                state.push_event(
                    "territory.overclaimed",
                    App::now(),
                    serde_json::json!({"faction_id": id, "previous_faction_id": previous, "world": claim.world, "chunk_x": claim.chunk_x, "chunk_z": claim.chunk_z}),
                );
                Ok(previous)
            })?;
            Ok(format!("Overclaimed this chunk from {previous}."))
        }
        "unclaim" => {
            app.mutate(player_id, "unclaim", |state| {
                let id = require_permission(app, state, player_id, RankPermission::Territory)?;
                state.unclaim_connected(&id, &claim)?;
                state.push_event(
                    "territory.unclaimed",
                    App::now(),
                    serde_json::json!({"faction_id": id, "world": claim.world, "chunk_x": claim.chunk_x, "chunk_z": claim.chunk_z}),
                );
                Ok(())
            })?;
            Ok("Chunk unclaimed.".into())
        }
        _ => unreachable!(),
    }
}

fn set_core(
    app: &App,
    player_id: &str,
    player: &pumpkin_plugin_api::Player,
) -> Result<String, String> {
    let (x, y, z) = player.get_position();
    let location = BlockLocation {
        world: player.get_world().get_id(),
        x: x.floor() as i32,
        y: y.floor() as i32,
        z: z.floor() as i32,
    };
    let initial_claims = App::initial_core_claims(&location);
    let now = App::now();
    let (faction_id, max_lives, replacement_cost) = {
        let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
        let faction_id = require_permission(app, &state, player_id, RankPermission::Core)?;
        let faction = &state.factions[&faction_id];
        if faction.has_active_core() {
            return Err("your faction already has an active physical core".into());
        }
        if faction.core_destroyed_at > now {
            return Err(format!(
                "a replacement core may be established after Unix time {}",
                faction.core_destroyed_at
            ));
        }
        let replacement_cost = if faction.core_lifecycle == CoreLifecycle::Destroyed {
            app.config.cores.replacement_cost.max(0)
        } else {
            0
        };
        if faction.bank < replacement_cost {
            return Err(format!(
                "the faction bank needs {replacement_cost} to establish a replacement core"
            ));
        }
        for claim in &initial_claims {
            if player
                .get_world()
                .get_chunk(claim.chunk_x, claim.chunk_z)
                .is_none()
            {
                return Err("all nine starting territory chunks must be loaded".into());
            }
            let border = player.get_world().get_border();
            let min_x = f64::from(claim.chunk_x * 16);
            let min_z = f64::from(claim.chunk_z * 16);
            if !border.contains(min_x, min_z) || !border.contains(min_x + 15.0, min_z + 15.0) {
                return Err(
                    "all nine starting territory chunks must fit inside the world border".into(),
                );
            }
            if state
                .zone_at(&claim.world, claim.chunk_x * 16 + 8, claim.chunk_z * 16 + 8)
                .is_some()
            {
                return Err("the 3x3 starting territory overlaps a server-owned zone".into());
            }
            if let Some(owner) = state.claim_owners.get(&claim.key())
                && owner != &faction_id
            {
                return Err(format!(
                    "the 3x3 starting territory overlaps faction {owner}"
                ));
            }
        }
        let center = location.claim();
        for (other_id, other) in &state.factions {
            if other_id == &faction_id {
                continue;
            }
            if let Some(other_core) = &other.physical_core {
                let other_claim = other_core.location.claim();
                if other_claim.world == center.world
                    && (other_claim.chunk_x - center.chunk_x).abs()
                        <= app.config.cores.enemy_core_distance_chunks
                    && (other_claim.chunk_z - center.chunk_z).abs()
                        <= app.config.cores.enemy_core_distance_chunks
                {
                    return Err("the proposed core is too close to another faction core".into());
                }
            }
        }
        (faction_id, app.core_max_lives(faction), replacement_cost)
    };

    let radius = app.config.cores.clearance_radius.max(0);
    let height = app.config.cores.clearance_height.max(0);
    for dy in 0..=height {
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let pos = BlockPos {
                    x: location.x + dx,
                    y: location.y + dy,
                    z: location.z + dz,
                };
                let block = player.get_world().get_block(pos);
                if !matches!(
                    block.name.as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                ) {
                    return Err(format!(
                        "core clearance is blocked at {}, {}, {} by {}",
                        pos.x, pos.y, pos.z, block.name
                    ));
                }
            }
        }
    }

    let block_pos = BlockPos {
        x: location.x,
        y: location.y,
        z: location.z,
    };
    let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
    if !player
        .get_world()
        .set_block_by_name(block_pos, "minecraft:beacon", flags)
    {
        return Err("Pumpkin could not place the beacon block".into());
    }
    let core = PhysicalCore {
        location: location.clone(),
        lives: max_lives,
        max_lives,
        last_hit_at: 0,
        established_at: now,
    };
    let result = app.mutate(player_id, "set-core", |state| {
        state.factions.get_mut(&faction_id).unwrap().bank -= replacement_cost;
        state.establish_core(&faction_id, core, initial_claims, now)?;
        state.push_event(
            "core.established",
            now,
            serde_json::json!({"faction_id": faction_id, "world": location.world, "x": location.x, "y": location.y, "z": location.z, "lives": max_lives}),
        );
        Ok(())
    });
    if let Err(error) = result {
        let _ = player
            .get_world()
            .set_block_by_name(block_pos, "minecraft:air", flags);
        return Err(error);
    }
    Ok(format!(
        "Physical faction core established with {max_lives} lives and a 3x3 territory{}.",
        if replacement_cost > 0 {
            format!(" for {replacement_cost}")
        } else {
            String::new()
        }
    ))
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
        if kind == UpgradeKind::Territory
            && let Some(core) = faction.physical_core.as_mut()
        {
            let max_lives =
                app.config.cores.starting_lives.saturating_add(
                    u32::from(level).saturating_mul(app.config.cores.lives_per_level),
                );
            core.max_lives = max_lives;
            core.lives = core
                .lives
                .saturating_add(app.config.cores.lives_per_level)
                .min(max_lives);
        }
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
                        second: None,
                    },
                );
            Ok(format!(
                "Zone '{id}' setup started. Tap the first corner block."
            ))
        }
        "convertzone" => {
            let id = FactionState::normalize(
                words
                    .get(1)
                    .ok_or("usage: /faction convertzone <legacy-zone>")?,
            );
            let zone = app
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .zones
                .get(&id)
                .cloned()
                .ok_or("zone not found")?;
            if zone.chunk_aligned {
                return Err("that zone is already chunk-aligned".into());
            }
            let first = Location {
                world: zone.world.clone(),
                x: f64::from(zone.min_x),
                y: 0.0,
                z: f64::from(zone.min_z),
            };
            let second = Location {
                world: zone.world,
                x: f64::from(zone.max_x),
                y: 0.0,
                z: f64::from(zone.max_z),
            };
            let buffer = match zone.kind {
                ZoneKind::Safe => app.config.zones.safe_buffer_chunks,
                ZoneKind::War => app.config.zones.war_buffer_chunks,
            }
            .max(0);
            let min_chunk_x = (first.x.floor() as i32).div_euclid(16) - buffer;
            let max_chunk_x = (second.x.floor() as i32).div_euclid(16) + buffer;
            let min_chunk_z = (first.z.floor() as i32).div_euclid(16) - buffer;
            let max_chunk_z = (second.z.floor() as i32).div_euclid(16) + buffer;
            let count =
                i64::from(max_chunk_x - min_chunk_x + 1) * i64::from(max_chunk_z - min_chunk_z + 1);
            app.zone_setup
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(
                    player_id.into(),
                    ZoneSetup {
                        id: id.clone(),
                        kind: zone.kind,
                        first: Some(first),
                        second: Some(second),
                    },
                );
            Ok(format!(
                "Legacy zone '{id}' would become chunks ({min_chunk_x}, {min_chunk_z}) through ({max_chunk_x}, {max_chunk_z}), {count} chunks including buffer. Use /faction zoneconfirm or /faction zonecancel."
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
        "zonecancel" => {
            app.zone_setup
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(player_id)
                .ok_or("no zone setup is awaiting confirmation")?;
            Ok("Zone setup cancelled.".into())
        }
        "zoneconfirm" => {
            let setup = app
                .zone_setup
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(player_id)
                .ok_or("no zone setup is awaiting confirmation")?;
            let first = setup
                .first
                .ok_or("the first corner has not been selected")?;
            let second = setup
                .second
                .ok_or("the opposite corner has not been selected")?;
            let buffer = match setup.kind {
                ZoneKind::Safe => app.config.zones.safe_buffer_chunks,
                ZoneKind::War => app.config.zones.war_buffer_chunks,
            };
            let count = app.mutate(player_id, "set-zone", |state| {
                state.set_chunk_zone(&setup.id, setup.kind, &first, &second, buffer)
            })?;
            Ok(format!(
                "Zone '{}' saved across {count} whole chunks.",
                setup.id
            ))
        }
        _ => unreachable!(),
    }
}

fn bank_command(app: &App, p: &str, words: &[&str]) -> Result<String, String> {
    let op = words.get(1).copied().unwrap_or("balance");
    if op == "balance" {
        let snapshot = app
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        let id = require_faction(&snapshot, p)?;
        let wallet = economy::balance(&app.config.economy, &snapshot, p)?;
        return Ok(format!(
            "Faction bank: {} • Wallet: {}",
            snapshot.factions[&id].bank, wallet
        ));
    }
    let amount = words
        .get(2)
        .ok_or("usage: /faction bank <deposit|withdraw> <amount>")?
        .parse::<i64>()
        .map_err(|_| "invalid amount")?;
    if amount <= 0 {
        return Err("amount must be positive".into());
    }
    let transaction_id = format!("cf:{}:{}:{}:{}", op, p, App::now(), amount);
    match op {
        "deposit" => {
            if app.config.economy.mode == config::EconomyMode::Standalone {
                return app.mutate(p, "bank-deposit", |state| {
                    let id = require_faction(state, p)?;
                    economy::debit(
                        &app.config.economy,
                        state,
                        p,
                        amount,
                        &transaction_id,
                        "faction_bank_deposit",
                    )?;
                    state.factions.get_mut(&id).unwrap().bank += amount;
                    Ok(format!("Deposited {amount}."))
                });
            }
            let mut external_state = FactionState::default();
            economy::debit(
                &app.config.economy,
                &mut external_state,
                p,
                amount,
                &transaction_id,
                "faction_bank_deposit",
            )?;
            if let Err(error) = app.mutate(p, "bank-deposit", |state| {
                let id = require_faction(state, p)?;
                state.factions.get_mut(&id).unwrap().bank += amount;
                Ok(())
            }) {
                let _ = economy::credit(
                    &app.config.economy,
                    &mut external_state,
                    p,
                    amount,
                    &format!("{transaction_id}:rollback"),
                    "faction_bank_deposit_rollback",
                );
                return Err(error);
            }
            Ok(format!("Deposited {amount}."))
        }
        "withdraw" => {
            app.mutate(p, "bank-withdraw", |state| {
                let id = require_permission(app, state, p, RankPermission::Economy)?;
                if state.factions[&id].bank < amount {
                    return Err("insufficient faction balance".into());
                }
                state.factions.get_mut(&id).unwrap().bank -= amount;
                if app.config.economy.mode == config::EconomyMode::Standalone {
                    economy::credit(
                        &app.config.economy,
                        state,
                        p,
                        amount,
                        &transaction_id,
                        "faction_bank_withdrawal",
                    )?;
                }
                Ok(())
            })?;
            if app.config.economy.mode == config::EconomyMode::External {
                let mut external_state = FactionState::default();
                if let Err(error) = economy::credit(
                    &app.config.economy,
                    &mut external_state,
                    p,
                    amount,
                    &transaction_id,
                    "faction_bank_withdrawal",
                ) {
                    let rollback = app.mutate(p, "bank-withdraw-rollback", |state| {
                        let id = require_faction(state, p)?;
                        state.factions.get_mut(&id).unwrap().bank += amount;
                        Ok(())
                    });
                    return Err(match rollback {
                        Ok(()) => error,
                        Err(rollback_error) => {
                            format!("{error}; faction-bank rollback also failed: {rollback_error}")
                        }
                    });
                }
            }
            Ok(format!("Withdrew {amount}."))
        }
        _ => Err("bank options: balance, deposit, withdraw".into()),
    }
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
        s.push_event(
            "war.declared",
            now,
            serde_json::json!({
                "war_id": id,
                "attacker": attacker,
                "defender": target,
                "forced": forced,
            }),
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
        let war_id = war.id.clone();
        let attacker = war.attacker.clone();
        let defender = war.defender.clone();
        if accept {
            war.status = WarStatus::Preparing;
            war.preparation_ends_at = Some(now + app.config.war.preparation_hours * 3600)
        } else {
            war.status = WarStatus::Declined
        }
        s.push_event(
            if accept {
                "war.accepted"
            } else {
                "war.declined"
            },
            now,
            serde_json::json!({"war_id": war_id, "attacker": attacker, "defender": defender}),
        );
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
            let war_id = war.id.clone();
            let attacker = war.attacker.clone();
            let defender = war.defender.clone();
            s.push_event(
                "war.started",
                now,
                serde_json::json!({"war_id": war_id, "attacker": attacker, "defender": defender}),
            );
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
        let mut started_wars = Vec::new();
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
                    started_wars.push((w.id.clone(), w.attacker.clone(), w.defender.clone()));
                    changed = true;
                    started = true;
                }
                _ => {}
            }
            let before = w.prisoners.len();
            w.prisoners.retain(|_, prisoner| prisoner.release_at > now);
            changed |= before != w.prisoners.len();
        }
        for (war_id, attacker, defender) in started_wars {
            candidate.push_event(
                "war.started",
                now,
                serde_json::json!({"war_id": war_id, "attacker": attacker, "defender": defender}),
            );
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
                let buffer = match setup.kind {
                    ZoneKind::Safe => self.0.config.zones.safe_buffer_chunks,
                    ZoneKind::War => self.0.config.zones.war_buffer_chunks,
                }
                .max(0);
                let first_x = (first.x.floor() as i32).div_euclid(16);
                let first_z = (first.z.floor() as i32).div_euclid(16);
                let second_x = (location.x.floor() as i32).div_euclid(16);
                let second_z = (location.z.floor() as i32).div_euclid(16);
                let width = (first_x - second_x).abs() + 1 + buffer * 2;
                let height = (first_z - second_z).abs() + 1 + buffer * 2;
                self.0
                    .zone_setup
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(
                        id,
                        ZoneSetup {
                            first: Some(first),
                            second: Some(location),
                            ..setup
                        },
                    );
                event.player.send_system_message(
                    TextComponent::text(&format!(
                        "Zone preview: {width}x{height} chunks (buffer {buffer}), {} total. Use /faction zoneconfirm or /faction zonecancel.",
                        width.saturating_mul(height)
                    )),
                    false,
                );
            } else {
                self.0
                    .zone_setup
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(
                        id,
                        ZoneSetup {
                            first: Some(location),
                            second: None,
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
            .territory_views
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&id);
        self.0
            .pending_territory
            .lock()
            .unwrap_or_else(|error| error.into_inner())
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
        server: Server,
        mut event: EventData<BlockBreakEvent>,
    ) -> EventData<BlockBreakEvent> {
        let world = event
            .player
            .as_ref()
            .map(|player| player.get_world().get_id());
        if let Some(world) = world
            && let Some((core_faction, _)) = self.0.core_at(
                &world,
                event.block_pos.x,
                event.block_pos.y,
                event.block_pos.z,
            )
        {
            event.cancelled = true;
            let Some(player) = &event.player else {
                return event;
            };
            let player_id = pid(player);
            let now = App::now();
            let result = self.0.mutate(&player_id, "core-hit", |state| {
                let attacker_faction = state
                    .player_faction
                    .get(&player_id)
                    .cloned()
                    .ok_or("only a faction enemy can damage a core")?;
                if attacker_faction == core_faction {
                    return Err("faction members cannot damage their own core".into());
                }
                if state.relation(&attacker_faction, &core_faction) != Relation::Enemy {
                    return Err("only an enemy faction can damage this core".into());
                }
                let core = state.factions[&core_faction]
                    .physical_core
                    .as_ref()
                    .ok_or("the core is no longer active")?;
                if core.last_hit_at.saturating_add(self.0.config.cores.hit_cooldown_seconds) > now {
                    return Err("the core is temporarily invulnerable after the last hit".into());
                }
                let faction_name = state.factions[&core_faction].name.clone();
                let core = state.factions.get_mut(&core_faction).unwrap().physical_core.as_mut().unwrap();
                core.last_hit_at = now;
                core.lives = core.lives.saturating_sub(1);
                let remaining = core.lives;
                state.push_event(
                    "core.attacked",
                    now,
                    serde_json::json!({"faction_id": core_faction, "faction_name": faction_name, "attacker_uuid": player_id, "remaining_lives": remaining, "world": world, "x": event.block_pos.x, "y": event.block_pos.y, "z": event.block_pos.z}),
                );
                let destroyed = remaining == 0;
                if destroyed {
                    state.destroy_core(
                        &core_faction,
                        now,
                        now.saturating_add(self.0.config.cores.replacement_cooldown_seconds),
                    )?;
                    state.push_event(
                        "core.destroyed",
                        now,
                        serde_json::json!({"faction_id": core_faction, "faction_name": faction_name, "attacker_uuid": player_id}),
                    );
                }
                Ok((remaining, destroyed))
            });
            match result {
                Ok((remaining, destroyed)) => {
                    player.send_system_message(
                        TextComponent::text(&format!(
                            "Core hit registered. Remaining lives: {remaining}."
                        )),
                        false,
                    );
                    for online in server.get_all_players() {
                        let online_id = pid(&online);
                        let member = self
                            .0
                            .state
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .player_faction
                            .get(&online_id)
                            == Some(&core_faction);
                        if member {
                            online.send_system_message(
                                TextComponent::text(&format!(
                                    "Your faction core was attacked! {remaining} lives remain."
                                )),
                                false,
                            );
                        }
                    }
                    if destroyed {
                        let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
                        let _ = player.get_world().set_block_by_name(
                            event.block_pos,
                            "minecraft:air",
                            flags,
                        );
                    }
                }
                Err(error) => player.send_system_message(TextComponent::text(&error), false),
            }
            return event;
        }
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
        let world = p.get_world().get_id();
        if self
            .0
            .core_clearance_owner(
                &world,
                event.block_pos.x,
                event.block_pos.y,
                event.block_pos.z,
            )
            .is_some()
        {
            event.cancelled = true;
            p.send_system_message(
                TextComponent::text("Blocks cannot be placed inside a faction core's clearance."),
                false,
            );
            return event;
        }
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
    s.push_event(
        "war.ended",
        now,
        serde_json::json!({
            "war_id": id,
            "winner": winner,
            "loser": loser,
            "reparations": paid,
        }),
    );
}

fn queue_territory_action(app: &App, player: &pumpkin_plugin_api::Player, claim: Claim) {
    let player_id = pid(player);
    let result = (|| -> Result<(String, String), String> {
        if player.get_world().get_id() != claim.world
            || player
                .get_world()
                .get_chunk(claim.chunk_x, claim.chunk_z)
                .is_none()
        {
            return Err("unknown or unloaded chunks are informational only".into());
        }
        let min_x = f64::from(claim.chunk_x * 16);
        let min_z = f64::from(claim.chunk_z * 16);
        let border = player.get_world().get_border();
        if !border.contains(min_x, min_z) || !border.contains(min_x + 15.0, min_z + 15.0) {
            return Err("out-of-border chunks are informational only".into());
        }
        let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
        let faction_id = require_permission(app, &state, &player_id, RankPermission::Territory)?;
        if state
            .zone_at(&claim.world, claim.chunk_x * 16 + 8, claim.chunk_z * 16 + 8)
            .is_some()
        {
            return Err("server-owned zones cannot be managed".into());
        }
        if state.factions.values().any(|faction| {
            faction
                .physical_core
                .as_ref()
                .is_some_and(|core| core.location.claim() == claim)
        }) {
            return Err("core chunks are informational and cannot be managed here".into());
        }
        let faction = &state.factions[&faction_id];
        let validate_expansion = || -> Result<(), String> {
            if faction.claims.len() >= app.core_claim_capacity(faction) {
                return Err("your core level cannot support another chunk".into());
            }
            if !faction
                .claims
                .iter()
                .any(|owned| owned.cardinally_adjacent(&claim))
            {
                return Err("only cardinal-adjacent chunks can be managed".into());
            }
            if app.config.cores.max_claim_distance_from_core > 0 {
                let core_claim = faction
                    .physical_core
                    .as_ref()
                    .ok_or("the faction core is not active")?
                    .location
                    .claim();
                let distance = (core_claim.chunk_x - claim.chunk_x).abs()
                    + (core_claim.chunk_z - claim.chunk_z).abs();
                if distance > app.config.cores.max_claim_distance_from_core {
                    return Err("the configured anti-corridor distance limit was reached".into());
                }
            }
            if state.factions.iter().any(|(other_id, other)| {
                other_id != &faction_id
                    && other.physical_core.as_ref().is_some_and(|core| {
                        let core_claim = core.location.claim();
                        core_claim.world == claim.world
                            && (core_claim.chunk_x - claim.chunk_x).abs()
                                <= app.config.cores.enemy_core_distance_chunks
                            && (core_claim.chunk_z - claim.chunk_z).abs()
                                <= app.config.cores.enemy_core_distance_chunks
                    })
            }) {
                return Err("the chunk is too close to another faction core".into());
            }
            Ok(())
        };
        let (action, description) = match state.claim_owner(&claim) {
            None => {
                validate_expansion()?;
                (
                    "claim",
                    format!(
                        "Claim wilderness chunk {}, {}",
                        claim.chunk_x, claim.chunk_z
                    ),
                )
            }
            Some(owner) if owner.id == faction_id => {
                if !state.removal_keeps_connected(&faction_id, &claim) {
                    return Err("releasing that chunk would disconnect territory".into());
                }
                (
                    "unclaim",
                    format!("Release owned chunk {}, {}", claim.chunk_x, claim.chunk_z),
                )
            }
            Some(owner) if state.relation(&faction_id, &owner.id) == Relation::Enemy => {
                validate_expansion()?;
                if !state.overclaimed(&owner.id)
                    || !state.removal_keeps_connected(&owner.id, &claim)
                {
                    return Err("that enemy chunk is not currently eligible for overclaim".into());
                }
                (
                    "overclaim",
                    format!(
                        "Overclaim chunk {}, {} from {}",
                        claim.chunk_x, claim.chunk_z, owner.name
                    ),
                )
            }
            Some(_) => return Err("that faction territory is not eligible for management".into()),
        };
        Ok((action.into(), description))
    })();
    match result {
        Ok((action, description)) => {
            app.pending_territory
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .insert(
                    player_id,
                    PendingTerritoryAction {
                        claim,
                        action,
                        expires_at: App::now().saturating_add(30),
                    },
                );
            player.send_system_message(
                TextComponent::text(&format!(
                    "{description}. Confirm within 30 seconds with /faction territoryconfirm, or cancel with /faction territorycancel."
                )),
                false,
            );
        }
        Err(error) => player.send_system_message(TextComponent::text(&error), false),
    }
}

fn pan_territory(
    app: &Arc<App>,
    server: &Server,
    player_id: &str,
    mut view: TerritoryView,
    dx: i32,
    dz: i32,
) {
    let now = App::now();
    if now
        < view
            .last_refresh_at
            .saturating_add(app.config.territory_ui.refresh_cooldown_seconds)
    {
        let player_id = player_id.to_string();
        server.schedule_delayed_task(1, move |server| {
            if let Some(player) = server
                .get_all_players()
                .into_iter()
                .find(|player| pid(player) == player_id)
            {
                player.send_system_message(
                    TextComponent::text("Please wait before panning the territory map again."),
                    false,
                );
            }
        });
        return;
    }
    let limit = app.config.territory_ui.max_pan_steps.max(0);
    view.offset_x = (view.offset_x + dx).clamp(-limit, limit);
    view.offset_z = (view.offset_z + dz).clamp(-limit, limit);
    view.last_refresh_at = now;
    let app = app.clone();
    let player_id = player_id.to_string();
    server.schedule_delayed_task(1, move |server| {
        if let Some(player) = server
            .get_all_players()
            .into_iter()
            .find(|player| pid(player) == player_id)
            && let Err(error) = ui::open_territory(&app, &player, view.clone())
        {
            player.send_system_message(TextComponent::text(&error), false);
        }
    });
}

struct Click(Arc<App>);
impl EventHandler<InventoryClickEvent> for Click {
    fn handle(
        &self,
        server: Server,
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
        event.cancelled = matches!(
            view.as_deref(),
            Some("main") | Some("mail") | Some("territory")
        );
        if view.as_deref() == Some("territory") {
            let territory_view = self
                .0
                .territory_views
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .get(&id)
                .cloned();
            if let Some(territory_view) = territory_view {
                match event.raw_slot {
                    45 => pan_territory(&self.0, &server, &id, territory_view, 0, -1),
                    46 => pan_territory(&self.0, &server, &id, territory_view, 0, 1),
                    47 => pan_territory(&self.0, &server, &id, territory_view, -1, 0),
                    48 => pan_territory(&self.0, &server, &id, territory_view, 1, 0),
                    slot @ 0..=44 if territory_view.management => {
                        if let Some(claim) = ui::territory_claim_for_slot(&territory_view, slot) {
                            queue_territory_action(&self.0, &event.player, claim);
                        }
                    }
                    _ => {}
                }
            }
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
        self.0
            .territory_views
            .lock()
            .unwrap_or_else(|error| error.into_inner())
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
        server: Server,
        event: EventData<BedrockFormResponseEvent>,
    ) -> EventData<BedrockFormResponseEvent> {
        self.0
            .forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&event.form_id);
        let territory = self
            .0
            .territory_forms
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&event.form_id);
        if let (Some(territory), Some(response)) = (territory, event.response_data.as_deref())
            && let Ok(index) = response.trim_matches('"').parse::<usize>()
            && let Some(action) = territory.actions.get(index).cloned()
        {
            match action {
                TerritoryFormAction::Pan(dx, dz) => {
                    pan_territory(
                        &self.0,
                        &server,
                        &pid(&event.player),
                        territory.view,
                        dx,
                        dz,
                    );
                }
                TerritoryFormAction::Inspect(claim) if territory.view.management => {
                    queue_territory_action(&self.0, &event.player, claim);
                }
                TerritoryFormAction::Inspect(_) => {}
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
