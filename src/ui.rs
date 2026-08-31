use crate::{
    app::{App, TerritoryFormAction, TerritoryFormView, TerritoryView, TradeView},
    config::RankPermission,
    domain::*,
};
use pumpkin_plugin_api::{
    ItemStack, Player, Screen,
    data_components::DataComponent,
    forms::SimpleFormBuilder,
    gui::Gui,
    scoreboard::{BedrockDisplaySlot, BedrockSortOrder, DisplaySlot, RenderType},
    text::TextComponent,
};

const DATA_COMPONENTS: &[DataComponent] = &[
    DataComponent::CustomData,
    DataComponent::MaxStackSize,
    DataComponent::MaxDamage,
    DataComponent::Damage,
    DataComponent::Unbreakable,
    DataComponent::UseEffects,
    DataComponent::CustomName,
    DataComponent::MinimumAttackCharge,
    DataComponent::DamageType,
    DataComponent::ItemName,
    DataComponent::ItemModel,
    DataComponent::Lore,
    DataComponent::Rarity,
    DataComponent::Enchantments,
    DataComponent::CanPlaceOn,
    DataComponent::CanBreak,
    DataComponent::AttributeModifiers,
    DataComponent::CustomModelData,
    DataComponent::TooltipDisplay,
    DataComponent::RepairCost,
    DataComponent::CreativeSlotLock,
    DataComponent::EnchantmentGlintOverride,
    DataComponent::IntangibleProjectile,
    DataComponent::Food,
    DataComponent::Consumable,
    DataComponent::UseRemainder,
    DataComponent::UseCooldown,
    DataComponent::DamageResistant,
    DataComponent::Tool,
    DataComponent::Weapon,
    DataComponent::AttackRange,
    DataComponent::Enchantable,
    DataComponent::Equippable,
    DataComponent::Repairable,
    DataComponent::Glider,
    DataComponent::TooltipStyle,
    DataComponent::DeathProtection,
    DataComponent::BlocksAttacks,
    DataComponent::PiercingWeapon,
    DataComponent::KineticWeapon,
    DataComponent::SwingAnimation,
    DataComponent::AdditionalTradeCost,
    DataComponent::StoredEnchantments,
    DataComponent::Dye,
    DataComponent::DyedColor,
    DataComponent::MapColor,
    DataComponent::MapId,
    DataComponent::MapDecorations,
    DataComponent::MapPostProcessing,
    DataComponent::ChargedProjectiles,
    DataComponent::BundleContents,
    DataComponent::PotionContents,
    DataComponent::PotionDurationScale,
    DataComponent::SuspiciousStewEffects,
    DataComponent::WritableBookContent,
    DataComponent::WrittenBookContent,
    DataComponent::Trim,
    DataComponent::DebugStickState,
    DataComponent::EntityData,
    DataComponent::BucketEntityData,
    DataComponent::BlockEntityData,
    DataComponent::Instrument,
    DataComponent::ProvidesTrimMaterial,
    DataComponent::OminousBottleAmplifier,
    DataComponent::JukeboxPlayable,
    DataComponent::ProvidesBannerPatterns,
    DataComponent::Recipes,
    DataComponent::LodestoneTracker,
    DataComponent::FireworkExplosion,
    DataComponent::Fireworks,
    DataComponent::Profile,
    DataComponent::NoteBlockSound,
    DataComponent::BannerPatterns,
    DataComponent::BaseColor,
    DataComponent::PotDecorations,
    DataComponent::Container,
    DataComponent::BlockState,
    DataComponent::Bees,
    DataComponent::SulfurCubeContent,
    DataComponent::Lock,
    DataComponent::ContainerLoot,
    DataComponent::BreakSound,
    DataComponent::VillagerVariant,
    DataComponent::WolfVariant,
    DataComponent::WolfSoundVariant,
    DataComponent::WolfCollar,
    DataComponent::FoxVariant,
    DataComponent::SalmonSize,
    DataComponent::ParrotVariant,
    DataComponent::TropicalFishPattern,
    DataComponent::TropicalFishBaseColor,
    DataComponent::TropicalFishPatternColor,
    DataComponent::MooshroomVariant,
    DataComponent::RabbitVariant,
    DataComponent::PigVariant,
    DataComponent::PigSoundVariant,
    DataComponent::CowVariant,
    DataComponent::CowSoundVariant,
    DataComponent::ChickenVariant,
    DataComponent::ChickenSoundVariant,
    DataComponent::ZombieNautilusVariant,
    DataComponent::FrogVariant,
    DataComponent::HorseVariant,
    DataComponent::PaintingVariant,
    DataComponent::LlamaVariant,
    DataComponent::AxolotlVariant,
    DataComponent::CatVariant,
    DataComponent::CatSoundVariant,
    DataComponent::CatCollar,
    DataComponent::SheepColor,
    DataComponent::ShulkerColor,
];

pub fn serialize_item(stack: &ItemStack) -> TradeItem {
    TradeItem {
        registry_key: stack.get_registry_key(),
        count: stack.get_count(),
        components: stack
            .get_components()
            .into_iter()
            .map(|component| TradeComponent {
                id: component.component as u16,
                value: component.value,
            })
            .collect(),
    }
}

pub fn deserialize_item(item: &TradeItem) -> ItemStack {
    let stack = ItemStack::new(&item.registry_key, item.count);
    for component in &item.components {
        if let Some(kind) = DATA_COMPONENTS.get(usize::from(component.id)).cloned() {
            stack.set_component(kind, &component.value);
        }
    }
    stack
}

fn item(id: &str, name: &str, lore: Vec<String>) -> ItemStack {
    let stack = ItemStack::new(id, 1);
    stack.set_custom_name(Some(TextComponent::text(name)));
    stack.set_lore(lore.into_iter().map(|v| TextComponent::text(&v)).collect());
    stack
}

fn localized(player: &Player, key: &str) -> TextComponent {
    TextComponent::custom("calabazafactions", key, &player.get_locale(), Vec::new())
}

struct ChunkMarker {
    item: &'static str,
    label: String,
    relation: String,
    actionable: bool,
}

fn chunk_marker(
    app: &App,
    player: &Player,
    state: &FactionState,
    player_id: &str,
    claim: &Claim,
    management: bool,
) -> ChunkMarker {
    if player.get_world().get_id() != claim.world
        || player
            .get_world()
            .get_chunk(claim.chunk_x, claim.chunk_z)
            .is_none()
    {
        return ChunkMarker {
            item: "minecraft:black_stained_glass_pane",
            label: "Unknown / Unloaded".into(),
            relation: "unknown".into(),
            actionable: false,
        };
    }
    let min_x = f64::from(claim.chunk_x * 16);
    let min_z = f64::from(claim.chunk_z * 16);
    let border = player.get_world().get_border();
    if !border.contains(min_x, min_z) || !border.contains(min_x + 15.0, min_z + 15.0) {
        return ChunkMarker {
            item: "minecraft:black_stained_glass_pane",
            label: "Outside World Border".into(),
            relation: "restricted".into(),
            actionable: false,
        };
    }
    let block_x = claim.chunk_x * 16 + 8;
    let block_z = claim.chunk_z * 16 + 8;
    if let Some(zone) = state.zone_at(&claim.world, block_x, block_z) {
        return match zone.kind {
            ZoneKind::Safe => ChunkMarker {
                item: "minecraft:lime_stained_glass_pane",
                label: format!("Safe Zone: {}", zone.id),
                relation: "safe_zone".into(),
                actionable: false,
            },
            ZoneKind::War => ChunkMarker {
                item: "minecraft:red_stained_glass_pane",
                label: format!("War Zone: {}", zone.id),
                relation: "war_zone".into(),
                actionable: false,
            },
        };
    }
    let own = state.player_faction.get(player_id);
    let can_expand = own
        .and_then(|id| state.factions.get(id).map(|faction| (id, faction)))
        .is_some_and(|(id, faction)| {
            let within_distance = faction.physical_core.as_ref().is_some_and(|core| {
                let core_claim = core.location.claim();
                app.config.cores.max_claim_distance_from_core <= 0
                    || (core_claim.chunk_x - claim.chunk_x).abs()
                        + (core_claim.chunk_z - claim.chunk_z).abs()
                        <= app.config.cores.max_claim_distance_from_core
            });
            let outside_other_cores = !state.factions.iter().any(|(other_id, other)| {
                other_id != id
                    && other.physical_core.as_ref().is_some_and(|core| {
                        let core_claim = core.location.claim();
                        core_claim.world == claim.world
                            && (core_claim.chunk_x - claim.chunk_x).abs()
                                <= app.config.cores.enemy_core_distance_chunks
                            && (core_claim.chunk_z - claim.chunk_z).abs()
                                <= app.config.cores.enemy_core_distance_chunks
                    })
            });
            faction.has_active_core()
                && faction.claims.len() < app.core_claim_capacity(faction)
                && faction
                    .claims
                    .iter()
                    .any(|owned| owned.cardinally_adjacent(claim))
                && within_distance
                && outside_other_cores
        });
    if let Some(owner) = state.claim_owner(claim) {
        let owner_core_chunk = owner
            .physical_core
            .as_ref()
            .is_some_and(|core| core.location.claim() == *claim);
        if own.is_some_and(|id| id == &owner.id) {
            return ChunkMarker {
                item: "minecraft:blue_stained_glass_pane",
                label: if owner_core_chunk {
                    format!("{} Core Territory", owner.name)
                } else {
                    format!("{} Territory", owner.name)
                },
                relation: "owned".into(),
                actionable: management
                    && !owner_core_chunk
                    && state.removal_keeps_connected(&owner.id, claim),
            };
        }
        let relation = own
            .map(|id| state.relation(id, &owner.id))
            .unwrap_or(Relation::Neutral);
        let (item, relation_name) = match relation {
            Relation::Ally => ("minecraft:cyan_stained_glass_pane", "ally"),
            Relation::Enemy => ("minecraft:red_stained_glass_pane", "enemy"),
            Relation::Truce => ("minecraft:yellow_stained_glass_pane", "truce"),
            Relation::Neutral => ("minecraft:yellow_stained_glass_pane", "neutral"),
        };
        return ChunkMarker {
            item,
            label: format!("{} Territory", owner.name),
            relation: relation_name.into(),
            actionable: management
                && relation == Relation::Enemy
                && !owner_core_chunk
                && state.overclaimed(&owner.id)
                && state.removal_keeps_connected(&owner.id, claim)
                && can_expand,
        };
    }
    ChunkMarker {
        item: "minecraft:white_stained_glass_pane",
        label: "Wilderness".into(),
        relation: "wilderness".into(),
        actionable: management && can_expand,
    }
}

pub fn territory_claim_for_slot(view: &TerritoryView, slot: i16) -> Option<Claim> {
    if !(0..45).contains(&slot) {
        return None;
    }
    let slot = i32::from(slot);
    let dx = slot.rem_euclid(9) - 4;
    let dz = slot.div_euclid(9) - 2;
    Some(Claim {
        world: view.origin.world.clone(),
        chunk_x: view.origin.chunk_x + view.offset_x + dx,
        chunk_z: view.origin.chunk_z + view.offset_z + dz,
    })
}

pub fn open_territory(app: &App, player: &Player, view: TerritoryView) -> Result<(), String> {
    let player_id = App::player_id(player);
    let state = app.state.lock().unwrap_or_else(|error| error.into_inner());
    let faction = state.faction_of(&player_id);
    let management = view.management
        && faction.is_some()
        && app.rank_allows(&state, &player_id, RankPermission::Territory);
    let faction_line = faction.map_or_else(
        || "No faction".to_string(),
        |faction| {
            format!(
                "{} • Core {:?} • Claims {}/{} • Core lives {}",
                faction.name,
                faction.core_lifecycle,
                faction.claims.len(),
                app.core_claim_capacity(faction),
                faction.physical_core.as_ref().map_or(0, |core| core.lives)
            )
        },
    );
    let title = if management {
        "Faction Territory"
    } else {
        "Faction Map"
    };

    if let Some(bedrock) = player.as_bedrock() {
        let mut preview = String::new();
        let mut actions = vec![
            TerritoryFormAction::Pan(0, -1),
            TerritoryFormAction::Pan(0, 1),
            TerritoryFormAction::Pan(-1, 0),
            TerritoryFormAction::Pan(1, 0),
            TerritoryFormAction::Recenter,
            TerritoryFormAction::Refresh,
            TerritoryFormAction::Status,
            TerritoryFormAction::ToggleManagement,
        ];
        let mut actionable = Vec::new();
        for slot in 0i16..45 {
            let claim = territory_claim_for_slot(&view, slot).unwrap();
            let marker = chunk_marker(app, player, &state, &player_id, &claim, management);
            let player_chunk = App::claim_at(player);
            preview.push(if claim == player_chunk {
                '@'
            } else {
                match marker.relation.as_str() {
                    "owned" => 'B',
                    "ally" => 'C',
                    "enemy" => 'R',
                    "neutral" | "truce" => 'Y',
                    "safe_zone" => 'L',
                    "war_zone" => 'W',
                    "wilderness" => '.',
                    _ => '#',
                }
            });
            if slot % 9 == 8 {
                preview.push('\n');
            }
            if marker.actionable {
                actionable.push((claim, marker.label));
            }
        }
        let mut form = SimpleFormBuilder::new(
            TextComponent::text(title),
            TextComponent::text(&format!(
                "{faction_line}\nOffset {}, {} (limit ±{})\n\n{preview}\n@ You • B Owned • C Ally • R Enemy • Y Neutral • L Safe • W War • . Wild • # Unknown",
                view.offset_x,
                view.offset_z,
                app.config.territory_ui.max_pan_steps
            )),
        )
        .button(TextComponent::text("↑ North"), None)
        .button(TextComponent::text("↓ South"), None)
        .button(TextComponent::text("← West"), None)
        .button(TextComponent::text("→ East"), None)
        .button(TextComponent::text("⌖ Recenter"), None)
        .button(TextComponent::text("↻ Refresh"), None)
        .button(TextComponent::text("Core Status"), None)
        .button(
            TextComponent::text(if management {
                "Switch to View Only"
            } else {
                "Switch to Management"
            }),
            None,
        );
        for (claim, label) in actionable.into_iter().take(20) {
            form = form.button(
                TextComponent::text(&format!("{label}\n{}, {}", claim.chunk_x, claim.chunk_z)),
                None,
            );
            actions.push(TerritoryFormAction::Inspect(claim));
        }
        let form_id = bedrock.open_form(form.build());
        let stored_view = TerritoryView { management, ..view };
        app.territory_forms
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(
                form_id,
                TerritoryFormView {
                    view: stored_view.clone(),
                    actions,
                },
            );
        app.territory_views
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(player_id, stored_view);
        return Ok(());
    }

    let gui = Gui::new(Screen::Generic9x6, TextComponent::text(title));
    gui.set_allow_grab_items(false);
    gui.set_allow_put_items(false);
    let player_chunk = App::claim_at(player);
    for slot in 0i16..45 {
        let claim = territory_claim_for_slot(&view, slot).unwrap();
        let marker = chunk_marker(app, player, &state, &player_id, &claim, management);
        let stack_id = if claim == player_chunk {
            "minecraft:player_head"
        } else {
            marker.item
        };
        let action = if marker.actionable {
            "Click to manage this chunk"
        } else {
            "Informational only"
        };
        gui.set_item(
            u32::try_from(slot).unwrap_or_default(),
            item(
                stack_id,
                &marker.label,
                vec![
                    format!("Chunk {}, {}", claim.chunk_x, claim.chunk_z),
                    format!("State: {}", marker.relation),
                    action.into(),
                ],
            ),
        );
    }
    for (slot, name) in [
        (45, "↑ North"),
        (46, "↓ South"),
        (47, "← West"),
        (48, "→ East"),
    ] {
        gui.set_item(
            slot,
            item(
                "minecraft:arrow",
                name,
                vec![format!(
                    "Current offset {}, {} • limit ±{}",
                    view.offset_x, view.offset_z, app.config.territory_ui.max_pan_steps
                )],
            ),
        );
    }
    gui.set_item(
        49,
        item(
            "minecraft:book",
            "Territory Guide",
            vec![
                faction_line,
                "Blue Owned • Red Enemy • Cyan Ally".into(),
                "Yellow Neutral • White Wilderness".into(),
                "Lime Safe • Red War • Black Unknown".into(),
                format!("View offset {}, {}", view.offset_x, view.offset_z),
            ],
        ),
    );
    gui.set_item(
        50,
        item(
            "minecraft:compass",
            "Recenter Map",
            vec!["Return the map to your current chunk.".into()],
        ),
    );
    gui.set_item(
        51,
        item(
            "minecraft:clock",
            "Refresh Map",
            vec!["Reload ownership, relations, zones, and status.".into()],
        ),
    );
    gui.set_item(
        52,
        item(
            "minecraft:beacon",
            "Core Status",
            vec!["Show core level, lives, clearance, and claim capacity.".into()],
        ),
    );
    gui.set_item(
        53,
        item(
            "minecraft:redstone_torch",
            if management {
                "Switch to View Only"
            } else {
                "Switch to Management"
            },
            vec![if management {
                "Disable territory action prompts.".into()
            } else {
                "Requires territory permission.".into()
            }],
        ),
    );
    drop(state);
    player.open_gui(gui);
    app.menus
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(player_id.clone(), "territory".into());
    app.territory_views
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(player_id, TerritoryView { management, ..view });
    Ok(())
}

pub fn update_scoreboard(app: &App, player: &Player) {
    let pid = App::player_id(player);
    let state = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let Some(faction) = state.faction_of(&pid) else {
        return;
    };
    let (power, claims, bank) = (
        faction
            .power
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        i32::try_from(faction.claims.len()).unwrap_or(i32::MAX),
        faction.bank.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
    );
    let first = app
        .scoreboards
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(pid);
    drop(state);

    if let Some(bedrock) = player.as_bedrock() {
        let board = bedrock.get_scoreboard();
        if first {
            board.add_objective("cfaction", "CalabazaFactions", BedrockSortOrder::Descending);
            board.set_display_slot(BedrockDisplaySlot::Sidebar, "cfaction");
        }
        board.update_score("Power", "cfaction", power);
        board.update_score("Claims", "cfaction", claims);
        board.update_score("Bank", "cfaction", bank);
    } else if let Some(java) = player.as_java() {
        let board = java.get_scoreboard();
        if first {
            board.add_objective(
                "cfaction",
                localized(player, "scoreboard.title"),
                RenderType::Integer,
            );
            board.set_display_slot(DisplaySlot::Sidebar, "cfaction");
        }
        board.update_score("Power", "cfaction", power);
        board.update_score("Claims", "cfaction", claims);
        board.update_score("Bank", "cfaction", bank);
    }
}

pub fn open_faction(app: &App, player: &Player) {
    let pid = App::player_id(player);
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let (title, body) = if let Some(f) = s.faction_of(&pid) {
        (
            format!("{} • Faction", f.name),
            format!(
                "{}\nPower {}/{}\nBank {}\nMembers {}\nClaims {}\nRole {:?}",
                if f.description.is_empty() {
                    "No description set."
                } else {
                    &f.description
                },
                f.power,
                f.max_power,
                f.bank,
                f.members.len(),
                f.claims.len(),
                f.members.get(&pid).unwrap_or(&Role::Recruit)
            ),
        )
    } else {
        (
            "CalabazaFactions".into(),
            "You are not in a faction. Use /faction create or /faction apply.".into(),
        )
    };
    drop(s);
    if let Some(bedrock) = player.as_bedrock() {
        let form = SimpleFormBuilder::new(TextComponent::text(&title), TextComponent::text(&body))
            .button(localized(player, "ui.mail"), None)
            .button(localized(player, "ui.claims"), None)
            .button(localized(player, "ui.wars"), None)
            .button(localized(player, "ui.trade"), None)
            .build();
        let id = bedrock.open_form(form);
        app.forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, "main".into());
    } else {
        let gui = Gui::new(Screen::Generic9x3, TextComponent::text(&title));
        gui.set_allow_grab_items(false);
        gui.set_allow_put_items(false);
        gui.set_item(
            4,
            item(
                "minecraft:player_head",
                "Faction Profile",
                body.lines().map(str::to_string).collect(),
            ),
        );
        gui.set_item(
            10,
            item(
                "minecraft:writable_book",
                "Faction Mail",
                vec!["Click or use /faction mail".into()],
            ),
        );
        gui.set_item(
            12,
            item(
                "minecraft:filled_map",
                "Claims & Territory",
                vec!["Use /faction map, claim, or unclaim".into()],
            ),
        );
        gui.set_item(
            14,
            item(
                "minecraft:iron_sword",
                "Diplomacy & War",
                vec!["Relations, readiness, and active wars".into()],
            ),
        );
        gui.set_item(
            16,
            item(
                "minecraft:chest",
                "Trade Inbox",
                vec!["Alliance trade deliveries".into()],
            ),
        );
        app.menus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(pid, "main".into());
        player.open_gui(gui);
    }
}

pub fn open_mail(app: &App, player: &Player) {
    let pid = App::player_id(player);
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let Some(fid) = s.player_faction.get(&pid) else {
        player.send_system_message(TextComponent::text("You are not in a faction."), false);
        return;
    };
    let mail = s.mail.get(fid).cloned().unwrap_or_default();
    if let Some(b) = player.as_bedrock() {
        let mut form = SimpleFormBuilder::new(
            TextComponent::text("Faction Mail"),
            TextComponent::text(&format!("{} message(s)", mail.len())),
        );
        for m in mail.iter().rev().take(20) {
            form = form.button(
                TextComponent::text(&format!("{}\n{}", m.subject, m.body)),
                None,
            );
        }
        let form_id = b.open_form(form.build());
        app.forms
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(form_id, "mail".into());
    } else {
        let gui = Gui::new(Screen::Generic9x6, TextComponent::text("Faction Mail"));
        gui.set_allow_grab_items(false);
        gui.set_allow_put_items(false);
        for (slot, m) in mail.iter().rev().take(54).enumerate() {
            gui.set_item(
                slot as u32,
                item(
                    "minecraft:paper",
                    &m.subject,
                    vec![m.body.clone(), format!("Message #{}", m.id)],
                ),
            );
        }
        app.menus
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(pid, "mail".into());
        player.open_gui(gui);
    }
}

pub fn open_trade_send(app: &App, player: &Player, target: &str) -> Result<(), String> {
    let pid = App::player_id(player);
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    if !app.rank_allows(&s, &pid, RankPermission::Trade) {
        return Err("your faction rank cannot send trade deliveries".into());
    }
    let mine = s
        .player_faction
        .get(&pid)
        .ok_or("you are not in a faction")?;
    if s.relation(mine, target) != Relation::Ally {
        return Err("trade requires a mutual alliance".into());
    }
    let capacity = s
        .factions
        .get(target)
        .map(|faction| app.trade_capacity(faction))
        .ok_or("target faction not found")?;
    if s.trade.get(target).map_or(0, Vec::len) + 27 > capacity {
        return Err("recipient trade inbox has fewer than 27 free slots".into());
    }
    drop(s);
    let gui = Gui::new(
        Screen::Generic9x3,
        TextComponent::text(&format!("Send goods to {target}")),
    );
    gui.set_allow_grab_items(true);
    gui.set_allow_put_items(true);
    let inventory = gui.get_inventory();
    app.trades.lock().unwrap_or_else(|e| e.into_inner()).insert(
        pid,
        TradeView::Send {
            target: target.into(),
            inventory,
        },
    );
    player.open_gui(gui);
    Ok(())
}

pub fn open_trade_inbox(app: &App, player: &Player) -> Result<(), String> {
    let pid = App::player_id(player);
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    if !app.rank_allows(&s, &pid, RankPermission::Trade) {
        return Err("your faction rank cannot open the trade inbox".into());
    }
    let fid = s
        .player_faction
        .get(&pid)
        .ok_or("you are not in a faction")?
        .clone();
    let items = s.trade.get(&fid).cloned().unwrap_or_default();
    drop(s);
    let gui = Gui::new(
        Screen::Generic9x6,
        TextComponent::text("Faction Trade Inbox"),
    );
    gui.set_allow_grab_items(true);
    gui.set_allow_put_items(false);
    for (slot, v) in items.iter().take(54).enumerate() {
        gui.set_item(slot as u32, deserialize_item(v));
    }
    let inventory = gui.get_inventory();
    app.trades.lock().unwrap_or_else(|e| e.into_inner()).insert(
        pid,
        TradeView::Inbox {
            faction: fid,
            inventory,
        },
    );
    player.open_gui(gui);
    Ok(())
}
