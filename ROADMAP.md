# CalabazaFaction Roadmap

## 0.1 — Foundation (current)

- Renamed plugin identity to `CalabazaFactions`.
- Safe-loading Rust/WASM plugin with serializable factions, roles, relations, claims, wars, mail, and trade state.
- Implemented create/delete, invite/application visibility rules, membership lookup API, claims/power checks, overclaim detection, prison prerequisites, and consensual/forced war state creation.
- Seed configuration and documented domain model.
- Pumpkin event wiring, persistence adapter, GUI forms/inventory views, and live teleport/combat enforcement remain future work.

## 0.2 — Identity and persistence

- Pumpkin command tree for the implemented create/delete/invite/apply/member/relation/claim/prison/war operations.
- Faction join/leave, ownership transfer, role permissions, and Faction Mail UI.
- JSON backend with migrations, atomic writes, backups, and audit events.
- `/faction` command tree and permission namespace `CalabazaFaction:command.faction`.

## 0.3 — Territory and protection

- Chunk claim/unclaim/map, faction home, wilderness and safe-zone policies.
- Server-authoritative block/container/entity interaction checks.
- Power ledger, death penalties, regeneration, over-claim rules, and admin inspection.

## 0.4 — Diplomacy and hardcore war

- Neutral/truce/ally/enemy relations and friendly-fire policy.
- War declarations, attack windows, raid shields, cooldowns, and grace periods.
- Faction core objectives, capture state machine, siege logs, and crash-safe resolution.

## 0.5 — MMO guild layer

- Faction bank, upgrades, permissions matrix, ranks, banners, homes, and member progression.
- Java inventory GUI and Bedrock forms with identical actions.
- Territory map overlays, scoreboards, notifications, and localization.

## 0.6 — Scale and operations

- SQLite/Postgres adapters, multi-server locking, snapshots, and repair tooling.
- Performance/load tests for large claim maps and concurrent raids.
- Admin dashboard, moderation/audit exports, and automated Pumpkin compatibility tests.

## Future research

- Compare power versus deaths-till-raidable balancing on test servers.
- Verify the stable Pumpkin event APIs needed for claims and combat enforcement.
- Define Java/Bedrock packet and UI fallbacks before committing to custom client assets.

## Note for future session agents

Treat `src/domain.rs` as the server-authoritative contract. Preserve atomic state transitions and the `FactionLookup` trait when wiring Pumpkin events, commands, Java inventories, Bedrock forms, teleport arenas, POW timers, and trade mailbox inventories. Do not block Tokio runtime threads; all persistence and UI callbacks must remain async-safe.
