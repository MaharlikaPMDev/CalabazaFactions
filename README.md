# CalabazaFactions

CalabazaFactions is a hardcore factions and guild-warfare plugin for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin). It is written in Rust and distributed as a WASI Preview 2 WebAssembly component.

The current release is **v0.3.1**. Features under **Planned for v0.4** are design commitments in the roadmap and are not available in the release yet.

## What is implemented in v0.3

### Factions and membership

- Public and invitation-only factions, applications, invitations, joining, leaving, kicking, leadership transfer, and disbanding.
- Leader, officer, veteran, member, and recruit roles.
- Configurable per-rank permissions for membership, territory, economy, diplomacy, war, homes, trade, and cores.
- Public faction information and editable descriptions.
- Persistent faction mail for applications, invitations, diplomacy, and war notices.

### Economy, power, and territory

- Player wallets and faction banks.
- Member-based power, death power loss, claim limits, protected chunk claims, unclaiming, and enemy overclaims.
- Faction homes and a relation/zone-aware territory map.
- Logical faction core locations with power, territory, vault, and shield upgrade trees.
- Component-safe faction banners copied from the leader's held item.

> In v0.3, `/faction setcore` saves the faction's central location for upgrades and faction identity. It does **not** place or protect a physical beacon. Physical beacon cores are planned for v0.4.

### Diplomacy, wars, and prisoners

- Neutral, truce, ally, and enemy relations with friendly-fire protection.
- Consensual war requests lasting up to 72 hours and immediate forced declarations.
- Configurable preparation, leader readiness, accelerated countdowns, and 30-minute arena battles.
- Named multi-arena rotation with multiple spawn positions for each faction side.
- War shields, declaration cooldowns, and post-war grace periods.
- POW capture during active wars, faction-bank ransom, configurable imprisonment, and prison-boundary enforcement.
- Reparations calculated from base cost, power difference, and troop count.
- One global pending, preparing, or active war lifecycle at a time.

### Trade, protection, and UI

- Alliance-only inventory trade with capacity checks and lossless persistence of Pumpkin item components, including names, lore, enchantments, custom data, and container contents.
- Independent safe zones and war zones.
- Claim/zone protection for building, containers, hoppers, pistons, explosions, fluids, buckets, and entity block grief.
- Localized English and Filipino messages.
- Java inventory pages, Bedrock Forms, faction sidebars, and map overlays.
- Informational menus that avoid unsafe nested GUI callbacks; actions shown in menus are performed through commands.

### Persistence and integrations

- Atomic JSON state writes, rolling backup recovery, corrupt-file preservation, bounded mail/audit history, and crash-recovery fixtures.
- A stable public `api.json` snapshot for external readers.
- Versioned same-server Pumpkin IPC lookups for player factions and relations.
- Event-driven war and POW timers without repeating schedulers or nested Tokio runtimes.

## Installation

1. Download `CalabazaFactions.wasm` and its checksum from the [latest release](https://github.com/MaharlikaPMDev/CalabazaFactions/releases/latest).
2. Optionally verify the artifact against `CalabazaFactions.wasm.sha256`.
3. Place `CalabazaFactions.wasm` in Pumpkin's `plugins` directory.
4. Start or restart the server.
5. CalabazaFactions creates its configuration and state under `plugins/data/CalabazaFactions/`.

The v0.3.1 artifact targets Pumpkin commit `e393751b8d441fda01710f242f6e4c610ea3c193`, pinned in `Cargo.toml`, and Minecraft Java 26.2-era Pumpkin builds. Pumpkin is under active development, so use the pinned-compatible server build when possible.

## Server setup guide

### 1. Review configuration

After the first startup, review `plugins/data/CalabazaFactions/config.toml`. Important sections control:

- Starting power, power per member, death loss, member limits, and claim rules.
- Starting wallet, reparations, and POW ransom.
- War request, preparation, battle, shield, cooldown, grace, and imprisonment durations.
- Mail, trade, and audit retention limits.
- Environmental claim protection.
- Core upgrade costs and bonuses.
- Permissions assigned to every faction rank.

Stop the server before editing generated state files. Treat `state.json` as private plugin data rather than a public integration format.

### 2. Configure war arenas

Run:

```text
/faction setarena <name>
```

Tap the first Team 1 and Team 2 spawn blocks when prompted. Add more spawn positions with:

```text
/faction addarenaspawn <name> <1|2>
```

Use `/faction arenas` to inspect configured arenas. Complete arenas rotate automatically between wars.

### 3. Configure zones

Create a safe or war zone with:

```text
/faction setzone <name> <safe|war>
```

Tap two opposite corners to complete the region. Safe zones take precedence if safe and war zones overlap.

### 4. Prepare factions for war

Each faction must establish a prison with `/faction setprison` before declaring or receiving a war. Faction leaders should also configure a home, logical core, banner, ranks, and diplomacy before entering combat.

## Player quick start

```text
/faction create <name> [public|private]
/faction setinfo <description>
/faction sethome
/faction setcore
/faction setprison
/faction setbanner
```

Then recruit members, deposit funds, and claim territory:

```text
/faction invite <player>
/faction bank deposit <amount>
/faction claim
/faction map
```

Use `/faction help [1|2|3]` in game for categorized help. The complete command reference is in [`docs/COMMANDS.md`](docs/COMMANDS.md). Primary aliases are `/faction` and `/f`.

## Gameplay overview

### Claims and overclaims

Stand inside a wilderness chunk and run `/faction claim`. Claim capacity is based on faction power and territory upgrades. Members with the configured territory permission can unclaim the current chunk.

`/faction overclaim` can take an enemy chunk only when that faction owns more claims than its available power permits. Building and environmental actions in protected territory are checked against faction membership, relations, and zone rules.

### Relations and war

Use `/faction relation <faction> <neutral|truce|ally|enemy>` to manage diplomacy. `/faction war <faction>` sends a consensual request; `/faction forcewar <faction>` begins forced-war preparation when all restrictions pass.

During an arena battle, attackers win by killing the defending leader. Defenders win if the battle timer expires. Eligible deaths can create POWs, and the losing faction may owe reparations from its available bank.

### Alliance trade

`/faction trade <allied faction>` opens a 27-slot outgoing shipment. `/faction tradeinbox` retrieves deliveries. The recipient must have sufficient mailbox capacity before a shipment is accepted.

## Commands and permissions

- `CalabazaFactions:command.faction` — standard player commands; allowed by default.
- `CalabazaFactions:command.admin` — arena and zone administration; operator level 3 by default.

Frequently used commands include:

- Membership: `/faction invite`, `apply`, `join`, `accept`, `kick`, `role`, `transfer`, and `leave`.
- Territory: `/faction claim`, `unclaim`, `overclaim`, `map`, `sethome`, `home`, `setcore`, and `core`.
- Economy: `/faction bank`, `/faction upgrade`, `/faction shield`, and `/faction paypow`.
- Diplomacy and combat: `/faction relation`, `war`, `forcewar`, `waraccept`, `wardecline`, and `ready`.
- Administration: `/faction setarena`, `addarenaspawn`, `delarena`, `setzone`, and `delzone`.

See [`docs/COMMANDS.md`](docs/COMMANDS.md) for syntax and behavior.

## Public integration API

The Rust domain exports `FactionLookup` for embedded consumers. Other Pumpkin plugins can send versioned JSON IPC requests to query a player's faction or the relation between two players. External processes and older Pumpkin hosts can read the atomically refreshed `api.json` snapshot.

Use UUIDs as stable player identifiers and ignore unknown response fields for forward compatibility. Never integrate against `state.json`. See [`docs/API.md`](docs/API.md) for request and response examples.

## Persistence and recovery

- `state.json` — authoritative private state.
- `state.json.bak` — previous successful state.
- `state.json.corrupt` — rejected primary state preserved during backup recovery.
- `api.json` — stable public read model.
- Temporary files — used during atomic replacement and cleaned through the persistence lifecycle.

If startup rejects the primary state, preserve all recovery files and inspect server logs before making manual changes.

## Planned for v0.4

v0.4 will center faction progression and territory around a physical beacon core. These features are **not implemented in v0.3.1**.

### Physical beacon cores

- `/faction setcore` will atomically validate clearance and conflicts, place a beacon, save its exact position, and establish territory.
- Newly created factions will remain in `AwaitingCore`. Until the founder successfully establishes a core, the faction cannot invite or accept members, accept applications, use public joining, or expand territory.
- A level-one core will begin with 10 configurable lives. Core levels will increase defensive strength and chunk capacity.
- A valid enemy break attempt will count as one hit while the physical break is cancelled. Hit cooldowns will prevent tool speed or packet spam from consuming lives instantly.
- At zero lives, the core and all faction claims will be removed without disbanding the faction. Its identity, members, ranks, relations, and history will remain.

### Event-driven clearance and safe reconciliation

- Block placement and other world-mutation events will prevent burying or modifying the protected core clearance area.
- One bounded reconciliation scheduler will repair exceptional state only; it will not determine attacks or destructive gameplay outcomes.
- Unloaded or unavailable chunks will be skipped without being forced to load. Ambiguous reads will be treated as unknown, never as core destruction.

### Strategic chunk expansion

- Establishing a core will grant a 3x3 territory centered on its chunk.
- Further claims will be selected one chunk at a time rather than automatically expanding in every direction.
- Every new chunk must share a north, south, east, or west edge with existing faction territory; diagonal contact will not qualify.
- Core level will control total chunk capacity, while connectivity and conflict rules will prevent islands and invalid claims.

### Cross-plugin event subscriptions

- CalabazaFactions will extend its IPC API with subscriptions for faction, core, raid, territory, and war events.
- Events will use stable versioned JSON envelopes, UUID identifiers, timestamps, topics, and monotonic sequence numbers.
- A bounded persistent journal and `events_since` cursor API will let listener plugins recover notifications missed during reloads or downtime.
- Subscriber and external-service failures will not block gameplay callbacks or roll back faction operations.

The complete implementation invariants and compatibility requirements are in [`ROADMAP.md`](ROADMAP.md).

## Known compatibility boundaries

- v0.3 uses the atomic JSON storage adapter. SQLite/Postgres and multi-server coordination remain future work.
- Java inventory and Bedrock form callbacks do not open nested screens. This avoids Pumpkin runtime re-entry associated with issue #1; use commands for actions displayed by informational menus.
- Some environmental Pumpkin events in the pinned API omit a world identifier. For piston, explosion, hopper, flow, and entity-grief checks, CalabazaFactions conservatively protects matching claimed/safe-zone coordinates in any world.
- Live Pumpkin integration tests are not yet part of CI.

## Building from source

Install the Rust targets/toolchains required by your platform, then run the equivalent of:

```text
cargo test --target x86_64-pc-windows-gnu --lib
cargo clippy --target wasm32-wasip2 --all-targets -- -D warnings
cargo build --release --locked --target wasm32-wasip2
```

The release component is written to `target/wasm32-wasip2/release/calabaza_factions.wasm`. The repository pins its Pumpkin API revision in `Cargo.toml` for reproducible compatibility.

## License

CalabazaFactions is released under the MIT license declared in `Cargo.toml`.
