# Command reference

## Identity and membership

- `/faction` — open the edition-appropriate faction page.
- `/faction create <name> [public|private]`
- `/faction disband`
- `/faction invite <player>`
- `/faction apply <faction>`
- `/faction join <faction>` — accept a valid invitation.
- `/faction accept <player>` — accept a public-faction application.
- `/faction leave`
- `/faction kick <player>`
- `/faction role <player> <officer|veteran|member|recruit>`
- `/faction transfer <player>`
- `/faction info [faction]`
- `/faction setinfo <description>` — set up to 160 characters.
- `/faction public` or `/faction private`
- `/faction help [1|2|3]` — colored, sectioned command help.

New and migrated factions must establish an active physical core before any recruitment path can add members. Leaders control ownership, transfers, and roles; all other actions follow the configured rank permission matrix.

## Economy and territory

- `/faction bank [balance]`
- `/faction bank deposit <amount>`
- `/faction bank withdraw <amount>`
- `/faction claim` — claim the current cardinal-adjacent chunk within core capacity.
- `/faction unclaim` — release the current chunk only when territory remains connected to the core.
- `/faction overclaim` — take the current enemy chunk only when its owner has more claims than power.
- `/faction map` — open the read-only 9x5 territory map.
- `/faction territory` — open the permission-gated territory management map.
- `/faction territoryconfirm` or `/faction territorycancel` — resolve a short-lived UI selection.
- `/faction sethome`
- `/faction home`
- `/faction setcore` — atomically place a beacon and grant the loaded, valid 3x3 starting territory.
- `/faction core` — teleport to the active physical core.
- `/faction setbanner` — copy the held item and all of its components as the faction banner.
- `/faction upgrade <power|territory|vault|shield>`

Core upgrades add maximum power, strategic chunk capacity/lives, trade capacity/reparation protection, or shield duration. Upgrade prices scale by level and are paid by the faction bank.

Java territory controls use `[↑][↓][←][→][BOOK][RECENTER][REFRESH][CORE STATUS][VIEW/MANAGE]`. Blue is owned, cyan ally, yellow neutral/truce, red enemy, white wilderness, lime safe zone, red war zone, and black unknown/unloaded. Bedrock receives equivalent native Form controls. All UI intents are deferred out of Pumpkin's click/Form callback, panning is bounded by configuration, and every mutation is revalidated at commit time.

## Diplomacy, war, and POWs

- `/faction relation <faction> <neutral|truce|ally|enemy>`
- `/faction setprison`
- `/faction war <faction>` — send a consensual request lasting 72 hours.
- `/faction forcewar <faction>` — skip consent and begin preparation.
- `/faction waraccept` or `/faction wardecline`
- `/faction ready` — the first leader shortens preparation to five minutes; both leaders ready starts immediately.
- `/faction shield` — activate a configurable shield, followed by a declaration cooldown.
- `/faction paypow <player>` — pay ransom from the faction bank.
- `/faction setarena [name]` — admin wizard: break the first Team 1 and Team 2 spawn blocks (the breaks are cancelled).
- `/faction addarenaspawn <arena> <1|2>` — append a spawn to one side's group.
- `/faction delarena <arena>`
- `/faction arenas`
- `/faction setzone <name> <safe|war>` — admin wizard: break two opposite corner blocks (the breaks are cancelled), then preview whole-chunk bounds and configured buffer.
- `/faction convertzone <legacy-zone>` — preview conversion of a preserved block-coordinate zone to buffered whole chunks.
- `/faction zoneconfirm` or `/faction zonecancel`
- `/faction delzone <name>`
- `/faction zones`

Attackers win by killing the defending leader within 30 minutes. If time expires, defenders win. Leader death settles reparations from the loser's available faction bank. Members killed during active war become POWs of the killer's faction for up to 24 hours unless ransomed.

Only one global war lifecycle may occupy the schedule. While any request is pending, a war is preparing, or a battle is active, all new consensual and forced declarations are rejected.

An active shield blocks declarations by or against its faction. After a shield expires, its cooldown prevents immediate reuse. War completion applies cooldown and grace state to both factions before either may enter another war.

## Mail and trade

- `/faction mail`
- `/faction trade <allied faction>` — open a 27-slot outgoing shipment.
- `/faction tradeinbox` — retrieve alliance deliveries.

Trade requires an ally relation. A sender is rejected unless the recipient has at least 27 free mailbox slots, preventing overflow loss.

Registry keys, counts, and every Pumpkin item data component are persisted. This includes enchantments, custom names, lore, custom data, container contents, and future components represented by the pinned API enum.

## Rank permissions

`config.toml` contains a matrix for leader, officer, veteran, member, and recruit. Each rank can independently receive membership, territory, economy, diplomacy, war, home, trade, and core permissions. Leadership transfer, role assignment, and disbanding remain leader-only ownership actions.

## Zone and environmental rules

Safe zones deny building, container use, PvP, explosions, fluids, pistons, and entity block grief. War zones allow PvP and building independently of faction claims. New zones are whole-chunk rectangles with explicit buffers; legacy block-coordinate zones are preserved. If zones overlap, safe zones take precedence.
