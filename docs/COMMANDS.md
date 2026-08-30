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

Leaders control ownership, transfers, and roles. Leaders and officers manage invitations, applications, claims, diplomacy, prisons, homes, and the bank.

## Economy and territory

- `/faction bank [balance]`
- `/faction bank deposit <amount>`
- `/faction bank withdraw <amount>`
- `/faction claim`
- `/faction unclaim`
- `/faction overclaim` — take the current enemy chunk only when its owner has more claims than power.
- `/faction map` — show a compact five-by-five chunk map.
- `/faction sethome`
- `/faction home`
- `/faction setcore`
- `/faction core`
- `/faction setbanner` — copy the held item and all of its components as the faction banner.
- `/faction upgrade <power|territory|vault|shield>`

Core upgrades add maximum power, bonus claim capacity, trade capacity/reparation protection, or shield duration. Upgrade prices scale by level and are paid by the faction bank.

## Diplomacy, war, and POWs

- `/faction relation <faction> <neutral|truce|ally|enemy>`
- `/faction setprison`
- `/faction war <faction>` — send a consensual request lasting 72 hours.
- `/faction forcewar <faction>` — skip consent and begin preparation.
- `/faction waraccept` or `/faction wardecline`
- `/faction ready` — the first leader shortens preparation to five minutes; both leaders ready starts immediately.
- `/faction shield` — activate a configurable shield, followed by a declaration cooldown.
- `/faction paypow <player>` — pay ransom from the faction bank.
- `/faction setarena [name]` — admin wizard: tap the first Team 1 and Team 2 spawn blocks.
- `/faction addarenaspawn <arena> <1|2>` — append a spawn to one side's group.
- `/faction delarena <arena>`
- `/faction arenas`
- `/faction setzone <name> <safe|war>` — admin wizard: tap two opposite corners.
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

Safe zones deny building, container use, PvP, explosions, fluids, pistons, and entity block grief. War zones allow PvP and building independently of faction claims. If zones overlap, safe zones take precedence.
