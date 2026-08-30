# CalabazaFactions

CalabazaFactions is a playable hardcore faction and guild-warfare plugin for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin), written in Rust and released as a WASI Preview 2 WebAssembly component.

## Features

- Public and invitation-only factions, applications, invitations, identity roles, leadership transfer, kicking, leaving, and disbanding.
- Persistent faction wallets/banks, member-based power limits, death power loss, chunk claims, protection, and enemy overclaims.
- Neutral, truce, ally, and enemy relations with friendly-fire protection.
- Consensual 72-hour war requests and immediate forced declarations, configurable preparation up to 12 hours, readiness, five-minute accelerated preparation, and 30-minute arena battles. Named arenas rotate automatically and support spawn groups for both sides.
- Explicit war shields, declaration cooldowns, and post-war grace periods, including shield core upgrades.
- POW capture during active wars, faction-bank ransom, configurable 24-hour imprisonment, and a ten-block prison boundary.
- War reparations based on base cost, power difference, and troop count.
- Persistent Faction Mail for applications, invitations, diplomacy, and war notices.
- Alliance-only inventory trade mailboxes with capacity protection and lossless persistence of all item data components.
- Independent safe/war zones plus container, hopper, piston, explosion, fluid, bucket, and entity-grief protection.
- Faction cores, component-safe banners, upgrade trees, and configurable rank permission matrices.
- Localized Java inventory/Bedrock Forms UI, faction scoreboards, and relation/zone-aware map overlays.
- Atomic JSON state and API snapshots, rolling backup recovery, bounded audit history, and a read-only `api.json` snapshot.
- Event-driven timers with no repeating scheduler or nested Tokio runtime calls.

## Installation

1. Download `CalabazaFactions.wasm` from the latest release.
2. Put it in Pumpkin's `plugins` directory.
3. Start the server. Configuration and state are created under `plugins/data/CalabazaFactions/`.
4. An administrator runs `/faction setarena <name>`, then taps the first Team 1 and Team 2 spawn blocks. Use `/faction addarenaspawn <name> <1|2>` for additional spawn-group positions. Multiple complete arenas rotate between wars.
5. Each faction must run `/faction setprison` before declaring or receiving a war.

The plugin targets the Pumpkin API commit pinned in `Cargo.toml` and Minecraft Java 26.2-era Pumpkin builds.

## Commands

Run `/faction help` in game. The full reference is in [`docs/COMMANDS.md`](docs/COMMANDS.md).

Primary aliases: `/faction` and `/f`.

Permissions:

- `CalabazaFactions:command.faction` — standard player command, allowed by default.
- `CalabazaFactions:command.admin` — administrative arena configuration, operator level 3 by default.

Faction essentials include `/faction sethome`, `/faction home`, `/faction setcore`, `/faction upgrade`, `/faction shield`, `/faction kick <player>`, `/faction info [faction]`, and `/faction setinfo <description>`.

## Public integration API

The Rust domain exports the `FactionLookup` trait for embedded consumers. Pumpkin plugins can query the same contract through versioned host IPC. Every successful mutation also atomically refreshes `plugins/data/CalabazaFactions/api.json`, containing player-to-faction mappings and safe public faction data. See [`docs/API.md`](docs/API.md).

Host IPC is preferred for live same-server lookups; `api.json` remains the compatibility contract for external processes and older Pumpkin hosts. Do not link to private implementation details.

## Persistence and recovery

- `state.json` is the authoritative state.
- `state.json.bak` is the previous successful state.
- `state.json.corrupt` preserves a rejected primary file when startup recovers from the backup.
- `api.json` is the public read model.
- Writes use a temporary file followed by rename.
- Audit history and mail retention are bounded by configuration.

## Build

```text
cargo +stable-x86_64-pc-windows-gnu test --target x86_64-pc-windows-gnu --lib
cargo +stable-x86_64-pc-windows-gnu clippy --target wasm32-wasip2 --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-gnu build --release --locked --target wasm32-wasip2
```

## Known compatibility boundaries

- The WASI release uses the atomic JSON storage adapter. SQLite/Postgres and cross-server coordination remain roadmap work because they require a callback-safe async/database host capability.
- Java inventory and Bedrock form callbacks do not open nested screens. This avoids Pumpkin runtime re-entry (#1); use the documented commands for actions shown by informational menus.
- Several environmental Pumpkin events currently omit a world identifier. For piston, explosion, hopper, flow, and entity-grief checks, CalabazaFactions conservatively protects matching claimed/safe-zone coordinates in any world.

See [`ROADMAP.md`](ROADMAP.md) for completed work and the next production-hardening tasks.
