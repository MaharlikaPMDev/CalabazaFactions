use crate::{
    app::{App, TradeView},
    domain::*,
};
use pumpkin_plugin_api::{
    ItemStack, Player, Screen, forms::SimpleFormBuilder, gui::Gui, text::TextComponent,
};

fn item(id: &str, name: &str, lore: Vec<String>) -> ItemStack {
    let stack = ItemStack::new(id, 1);
    stack.set_custom_name(Some(TextComponent::text(name)));
    stack.set_lore(lore.into_iter().map(|v| TextComponent::text(&v)).collect());
    stack
}

pub fn open_faction(app: &App, player: &Player) {
    let pid = App::player_id(player);
    let s = app.state.lock().unwrap_or_else(|e| e.into_inner());
    let (title, body) = if let Some(f) = s.faction_of(&pid) {
        (
            format!("{} • Faction", f.name),
            format!(
                "Power {}/{}\nBank {}\nMembers {}\nClaims {}\nRole {:?}",
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
            .button(TextComponent::text("Faction Mail"), None)
            .button(TextComponent::text("Claims / Map"), None)
            .button(TextComponent::text("Diplomacy / Wars"), None)
            .button(TextComponent::text("Trade Inbox"), None)
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
    let mine = s
        .player_faction
        .get(&pid)
        .ok_or("you are not in a faction")?;
    if s.relation(mine, target) != Relation::Ally {
        return Err("trade requires a mutual alliance".into());
    }
    if s.trade.get(target).map_or(0, Vec::len) + 27 > app.config.storage.trade_slots {
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
        gui.set_item(slot as u32, ItemStack::new(&v.registry_key, v.count));
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
