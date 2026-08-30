use crate::{
    app::{App, TradeView},
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
