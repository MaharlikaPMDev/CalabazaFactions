# Changelog

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
