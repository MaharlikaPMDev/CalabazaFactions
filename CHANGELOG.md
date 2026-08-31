# Changelog

## 0.4.2

- Changed arena spawn and server-zone coordinate selection from block interaction to cancelled block-break events, preventing one right-click from recording both setup positions.

## 0.4.1

- Built against Pumpkin commit `c01ddb7b65b0be9641eb2273740ce8e1a04b7807`.
- Fixed `/faction setcore` rejecting Pumpkin block names such as `Air`; air, cave air, and void air now match case-insensitively with or without a namespace.
- Replaced separate clearance radius/height settings with `clearance_outward_blocks = 4`, covering four blocks horizontally and upward from the beacon while leaving the support area below exempt.
- Removed all host/UI work from inventory and Bedrock Form callbacks. Callbacks now only cancel and enqueue bounded intents for a pre-registered tick task, preventing GUI-close freezes.
- Filled territory slots 50–53 with Recenter, Refresh, Core Status, and View/Management Toggle controls on Java and equivalent Bedrock Form actions.

## 0.4.0

- Replaced logical cores with atomic physical beacon cores, a 3x3 initial territory, configurable lives and hit cooldowns, enemy-hit damage, destruction snapshots, claim teardown, replacement cooldowns, and loaded-chunk-only reconciliation.
- Added the `AwaitingCore`, `Active`, and `Destroyed` lifecycle and enforced the active-core recruitment/expansion gate in the domain layer.
- Added strategic cardinal-adjacent chunk expansion, core-level capacity, connected-territory unclaim checks, world-border/zone/load validation, enemy-core spacing, and indexed claim/core lookup.
- Added `/faction map` and `/faction territory` with a 9x5 pane map, player-head position, the finalized `[↑][↓][←][→][BOOK]` bottom row, configurable panning bounds, safe confirmations, and Bedrock Forms.
- Made new safe/war zones chunk-aligned with explicit preview, confirmation, and configurable buffers while preserving legacy block-coordinate zones.
- Added a persistent, monotonic faction event journal; IPC subscriptions, live topic delivery, cursor recovery, and faction/core/territory/war events.
- Added standalone and external economy modes plus the versioned CalabazaBank IPC contract. In external mode, CalabazaFactions no longer owns the global player balance.
- Expanded the public API snapshot to schema 4 and added v0.4 migration and regression coverage.
- Retained the non-reentrant deferred GUI pattern from the #1 fix and expanded the formatted help from #2 for the new territory commands.
- Built against Pumpkin commit `20c51d346e33f1f485f401b6d159a9f0881ec1af`.

## 0.3.1

- Rebuilt against Pumpkin commit `e393751b8d441fda01710f242f6e4c610ea3c193` to restore WASM plugin ABI compatibility after Pumpkin's event and command API update.

## 0.3.0

- Added lossless trade item persistence for every Pumpkin data component, including names, lore, enchantments, container contents, and custom data.
- Added named multi-arena rotation, per-side spawn groups, war shields, declaration cooldowns, and post-war grace periods.
- Added independent safe/war zones and protection for containers, hoppers, pistons, explosions, fluids, buckets, and entity block grief.
- Added faction cores, power/territory/vault/shield upgrade trees, component-safe faction banners, and configurable per-rank permission matrices.
- Added English and Filipino UI translations, faction sidebars, and a relation/zone-aware nine-by-nine territory map.
- Added a versioned host IPC lookup API while preserving `api.json` schema compatibility.
- Added schema-v2 migration, corrupt-primary backup recovery, atomic API writes, a 10,000-claim load fixture, and crash-recovery tests.
- Fixed #1 by making inventory/form callbacks non-reentrant; menu clicks are informational and no longer open another GUI from inside a click callback.
- Fixed #2 with colored, sectioned, three-page help and one command group per line.

## 0.2.1

- Replaced the single-position arena command with a persistent Team 1/Team 2 block-tap setup wizard.
- Enforced one global pending, preparing, or active war at a time.
- Added `/faction info` and `/faction setinfo`; documented the existing home and kick commands.

## 0.2.0

- Renamed the project and plugin to CalabazaFactions.
- Added playable faction identity, membership, economy, power, claims, overclaims, diplomacy, wars, POWs, mail, trade, Java inventory UI, and Bedrock Forms UI.
- Added atomic persistence, backups, audit history, and public integration snapshot.
- Added strict event-driven war and POW timing without scheduler/runtime blocking.
