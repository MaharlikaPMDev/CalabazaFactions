# CalabazaFaction — WORK IN PROGRESS

CalabazaFaction is a planned hardcore faction and guild warfare system for [PumpkinMC](https://github.com/Pumpkin-MC/Pumpkin). This public repository currently contains a safe-loading Rust/WASM skeleton, configuration defaults, design notes, and the implementation roadmap.

## Current status

Version `0.1.0` only verifies that the plugin can load cleanly. No faction commands, claims, combat rules, or persistence are enabled yet.

The design is informed by established Factions conventions: chunk-based territory, member power, over-claim vulnerability, diplomacy relations, faction homes, raid windows, and deaths-til-raidable concepts. See the [research notes](docs/design.md).

## Planned scope

- Factions and guild roles with invite, kick, promote, demote, and transfer ownership flows.
- Chunk claims with wilderness, safe-zone, war-zone, and faction-territory access policies.
- Personal/faction power, over-claim protection, death penalties, and configurable regeneration.
- Neutral, truce, ally, and enemy relations with friendly-fire policies.
- Hardcore raids with attack windows, shields, core objectives, siege logs, and rollback-safe state.
- Faction bank, upgrades, homes, map/territory visualization, GUI, audit log, and admin tooling.
- Java and Bedrock-compatible presentation with vanilla fallbacks.

## Build

```text
cargo +stable-x86_64-pc-windows-gnu build --release --target wasm32-wasip2
```

The resulting WASM will be packaged as a Pumpkin plugin once the first gameplay milestone is complete.

## Configuration

Defaults are in [`config/factions.toml`](config/factions.toml). They are deliberately disabled until the corresponding systems are implemented and tested.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for milestones, invariants, and future research tasks.
