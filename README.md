# CalabazaFactions

CalabazaFactions is a hardcore factions and guild-warfare plugin for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin), written in Rust and distributed as a WASI Preview 2 WebAssembly component.

The current release is **v0.4.0**. It targets Pumpkin commit `20c51d346e33f1f485f401b6d159a9f0881ec1af` and the current v0.1 plugin ABI.

## Highlights

- Physical beacon cores with clearance, lives, upgrades, hit cooldowns, destruction, replacement cooldowns, and safe reconciliation.
- Strategic chunk territory: a 3x3 starting claim followed by individually selected, cardinal-adjacent chunks.
- Java inventory and Bedrock Forms territory maps with separate viewing and management commands.
- Public/private factions, configurable ranks, recruitment, faction mail, banks, homes, banners, diplomacy, arenas, wars, POWs, ransom, and reparations.
- Safe/war zones, claim protection, alliance trade, localization, scoreboards, atomic persistence, and versioned IPC integrations.
- Standalone player balances or an external economy owned by the future `CalabazaBank` plugin.

## Installation

1. Download `CalabazaFactions.wasm` and `CalabazaFactions.wasm.sha256` from the [latest release](https://github.com/MaharlikaPMDev/CalabazaFactions/releases/latest).
2. Optionally verify the checksum.
3. Place the WASM file in Pumpkin's `plugins` directory.
4. Start Pumpkin. Configuration and state are created under `plugins/data/CalabazaFactions/`.

Pumpkin's WASM ABI is evolving. v0.4.0 is pinned to the commit above so the exported `handle-event` signature matches the server API used to build the artifact.

## First server setup

Review `plugins/data/CalabazaFactions/config.toml` after first startup. The bundled defaults are in [`config/factions.toml`](config/factions.toml).

Important settings include:

- `[cores]`: lives, claim capacity, clearance, hit/replacement cooldowns and cost, reconciliation batch size, enemy-core spacing, and an optional anti-corridor distance cap (`0` disables the cap).
- `[territory_ui]`: maximum map pan distance, five chunks by default.
- `[zones]`: explicit safe-zone and war-zone chunk buffers.
- `[economy]`: `standalone` or `external`; the external provider defaults to `CalabazaBank`.
- `[ipc]`: event journal retention and live-delivery interval.
- `[ranks.*]`: membership, territory, economy, diplomacy, war, home, trade, and core permissions.

Configure at least one complete war arena:

```text
/faction setarena <name>
/faction addarenaspawn <name> <1|2>
/faction arenas
```

Create server zones by selecting two corners and confirming the preview:

```text
/faction setzone <name> <safe|war>
/faction zoneconfirm
```

New zones cover whole chunks. The preview reports selected and buffered bounds; safe zones take precedence over overlapping war zones. Existing pre-v0.4 block-coordinate zones remain unchanged during migration.

Use `/faction convertzone <legacy-zone>` to preview an explicit whole-chunk conversion without changing the old zone until `/faction zoneconfirm`.

## Player quick start

Create a faction, then establish its most important structure:

```text
/faction create <name> [public|private]
/faction setcore
```

`/faction setcore` places a physical beacon at the player's block position after validating clearance, all nine loaded starting chunks, ownership, zones, world borders, and nearby faction cores. It then grants the centered 3x3 territory atomically.

A new faction is `AwaitingCore`. Until its beacon is established it cannot invite, accept applications or invitations, use public joining, or expand. Pre-v0.4 factions keep their identity, members, economy, and history but migrate to this same state without silently placing blocks. Their old non-core claims are deactivated and retained in the recoverable claim snapshot before the new 3x3 is established.

Continue setup with:

```text
/faction setinfo <description>
/faction sethome
/faction setprison
/faction setbanner
/faction bank deposit <amount>
```

Use `/faction help [1|2|3]` for colored, sectioned in-game help and [`docs/COMMANDS.md`](docs/COMMANDS.md) for the complete reference.

## Physical core rules

A level-one core starts with 10 lives by default. Blocks cannot be placed in its configured clearance volume. Claim and environmental handlers protect the beacon; a bounded scheduler only repairs unexpectedly missing beacons in already loaded chunks and never deals damage.

A valid enemy break attempt is cancelled and counts as one hit after the global core-hit cooldown. At zero lives:

- the physical beacon is removed;
- every owned chunk is revoked through the ownership index;
- the faction becomes `Destroyed` and cannot recruit or expand;
- faction identity, members, ranks, relations, history, bank, and a recoverable claim snapshot remain intact.

After the configurable replacement cooldown, an authorized member can establish a replacement with `/faction setcore`.

## Strategic territory and map

After the initial 3x3, each new chunk is selected separately. It must share a north, south, east, or west edge with existing territory; diagonal contact is insufficient. Capacity comes from core level. Loaded state, world border, zones, ownership, capacity, adjacency, core spacing, and connectivity are revalidated when a change commits.

- `/faction map` opens the read-only territory view.
- `/faction territory` opens management for ranks with territory permission.
- `/faction claim`, `/faction overclaim`, and `/faction unclaim` operate on the current chunk through the same domain rules.

Java uses a 9x5 pane map. The player head marks the viewer's current chunk while preserving ownership details.
The arrows pan within the configured limit. The book contains faction/core capacity and this legend:

- blue: owned territory
- cyan: ally
- yellow: neutral or truce
- red: enemy
- white: wilderness
- lime: safe zone
- red: war zone
- black: unknown, unloaded, restricted, or outside the usable view

Bedrock uses a native Form with the same 9x5 information, direction controls, and a compact list of actionable chunks. UI selections create a short-lived confirmation; commit-time validation remains authoritative. GUI callbacks never open nested screens directly, preserving the non-reentrant fix for issue #1.

## Economy modes

In `standalone` mode, CalabazaFactions maintains local player wallets and faction banks.

In `external` mode, the global player balance belongs to another plugin. CalabazaFactions sends versioned debit, credit, balance, and health requests to the configured provider while retaining only its faction-specific bank. Deposits and withdrawals use transaction IDs and compensation paths so a failed second step does not silently lose funds.

The future CalabazaBank wire contract is documented in [`docs/API.md`](docs/API.md). External mode can be configured before that plugin exists; startup warns that the provider is unavailable and economy operations fail safely.

## Integrations and persistence

Other Pumpkin plugins can query factions/relations and subscribe to faction, core, territory, war, and reserved raid topics over host IPC. Events use a stable schema, monotonic sequence numbers, bounded persistence, and `events_since` recovery. See [`docs/API.md`](docs/API.md).

Files under the plugin data directory:

- `state.json`: authoritative private state, schema 4
- `state.json.bak`: previous successful state
- `state.json.corrupt`: rejected primary preserved during recovery
- `api.json`: stable public read model
- `config.toml`: generated configuration

Writes are atomic and preserve a rolling backup. Integrate through IPC or `api.json`, never through private `state.json`.

## Building

```text
cargo test --lib
cargo clippy --target wasm32-wasip2 --all-targets -- -D warnings
cargo build --release --locked --target wasm32-wasip2
```

The artifact is `target/wasm32-wasip2/release/calabaza_factions.wasm`.

## Known boundaries and upcoming work

- JSON is the portable v0.4 storage backend. SQLite/Postgres and multi-server coordination remain later work.
- Custom Minecraft map-data rendering is deferred; territory management does not depend on packets or clickable map pixels.
- `raid.*` IPC topics are reserved for a future authoritative raid lifecycle; core and war attack events are available now.
- Live server integration tests are not yet part of CI, so test a release artifact on a staging Pumpkin server before production rollout.

## License

CalabazaFactions is released under the MIT license declared in `Cargo.toml`.
