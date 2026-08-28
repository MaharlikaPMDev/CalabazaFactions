# CalabazaFactions

CalabazaFactions is a playable hardcore faction and guild-warfare plugin for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin), written in Rust and released as a WASI Preview 2 WebAssembly component.

## Features

- Public and invitation-only factions, applications, invitations, identity roles, leadership transfer, kicking, leaving, and disbanding.
- Persistent faction wallets/banks, member-based power limits, death power loss, chunk claims, protection, and enemy overclaims.
- Neutral, truce, ally, and enemy relations with friendly-fire protection.
- Consensual 72-hour war requests and immediate forced declarations, configurable preparation up to 12 hours, leader readiness, five-minute accelerated preparation, and 30-minute arena battles.
- POW capture during active wars, faction-bank ransom, configurable 24-hour imprisonment, and a ten-block prison boundary.
- War reparations based on base cost, power difference, and troop count.
- Persistent Faction Mail for applications, invitations, diplomacy, and war notices.
- Alliance-only inventory trade mailboxes with capacity protection.
- Java inventory pages and Bedrock Forms UI.
- Atomic JSON state, rolling backup, bounded audit history, and a read-only `api.json` snapshot.
- Event-driven timers with no repeating scheduler or nested Tokio runtime calls.

## Installation

1. Download `CalabazaFactions.wasm` from the latest release.
2. Put it in Pumpkin's `plugins` directory.
3. Start the server. Configuration and state are created under `plugins/data/CalabazaFactions/`.
4. An administrator must stand in the battle arena and run `/faction setarena` before wars can begin.
5. Each faction must run `/faction setprison` before declaring or receiving a war.

The plugin targets the Pumpkin API commit pinned in `Cargo.toml` and Minecraft Java 26.2-era Pumpkin builds.

## Commands

Run `/faction help` in game. The full reference is in [`docs/COMMANDS.md`](docs/COMMANDS.md).

Primary aliases: `/faction` and `/f`.

Permissions:

- `CalabazaFactions:command.faction` — standard player command, allowed by default.
- `CalabazaFactions:command.admin` — administrative arena configuration, operator level 3 by default.

## Public integration API

The Rust domain exports the `FactionLookup` trait for embedded consumers. Every successful mutation also atomically refreshes `plugins/data/CalabazaFactions/api.json`, containing player-to-faction mappings and safe public faction data. See [`docs/API.md`](docs/API.md).

Pumpkin's current WASM API does not yet expose a live inter-plugin service registry. Separate WASM plugins should treat the JSON snapshot as the compatibility contract until Pumpkin adds cross-plugin calls; do not link to private implementation details.

## Persistence and recovery

- `state.json` is the authoritative state.
- `state.json.bak` is the previous successful state.
- `api.json` is the public read model.
- Writes use a temporary file followed by rename.
- Audit history and mail retention are bounded by configuration.

## Build

```text
cargo +stable-x86_64-pc-windows-gnu test --target x86_64-pc-windows-gnu --lib
cargo +stable-x86_64-pc-windows-gnu clippy --target wasm32-wasip2 --all-targets -- -D warnings
cargo +stable-x86_64-pc-windows-gnu build --release --locked --target wasm32-wasip2
```

## Known compatibility boundary

Trade mailboxes preserve the vanilla registry key and stack count. Arbitrary custom components, names, lore, and enchantment metadata are not serialized by Pumpkin's current stable item API and should not be deposited until component-safe persistence is added.

See [`ROADMAP.md`](ROADMAP.md) for completed work and the next production-hardening tasks.
