# CalabazaFactions Roadmap

## v0.2 — Playable factions release

- [x] Rename plugin and repository to `CalabazaFactions`.
- [x] Creation, deletion, invitations, applications, and public/private factions.
- [x] Leader, officer, veteran, member, and recruit identity roles.
- [x] Faction bank, player wallet, power, claims, protection, and enemy overclaims.
- [x] Neutral, truce, ally, and enemy diplomacy with friendly-fire protection.
- [x] Faction Mail for applications, invitations, diplomacy, and wars.
- [x] Consensual and forced wars, 72-hour requests, preparation, readiness, arena teleport, and 30-minute battles.
- [x] Persistent two-spawn arena setup wizard and single global war scheduling lock.
- [x] Faction home, kick, public info, and editable 160-character descriptions.
- [x] POW capture, ransom, 24-hour release, and prison-boundary enforcement.
- [x] Power/troop-based war reparations.
- [x] Alliance-only item trade mailbox with capacity protection.
- [x] Java inventory faction page and Bedrock Forms page.
- [x] Atomic JSON persistence, backup, audit log, and public API snapshot.
- [x] Event-driven implementation without a repeating scheduler or Tokio blocking.

## v0.3 — Production hardening

- [x] Persist full item data components for trade mail, including enchantments, names, lore, and custom data.
- [x] Add configurable multi-arena selection and spawn groups per faction side.
- [x] Add explicit war shield/cooldown and post-war grace-period state.
- [x] Add safe-zone and war-zone administration independent of faction claims.
- [x] Add container, piston, explosion, fluid, and entity-grief protection coverage.
- [x] Add faction cores, upgrade trees, banners, and configurable rank permission matrices.
- [x] Add localized UI messages, Java/Bedrock scoreboards, and richer relation/zone map overlays.
- [ ] Add GUI actions for every command after Pumpkin exposes a callback-safe deferred GUI dispatch path. Menus remain informational to prevent #1 from recurring.
- [ ] Add SQLite/Postgres adapters and multi-server coordination.
- [x] Add load tests and crash-recovery fixtures.
- [ ] Add live Pumpkin integration tests to CI.
- [x] Adopt Pumpkin's host-backed IPC API for versioned faction/relation lookups.

## Note for future session agents

`src/domain.rs` is the server-authoritative contract. Preserve atomic transitions, UUID-based identity, the `FactionLookup` trait, and `api.json` schema compatibility. Never use `block_on`, nested Tokio runtimes, or Pumpkin's scheduler from ticker/runtime callbacks. Timed war and POW transitions are intentionally processed from commands and player events. Any new inventory persistence must round-trip every item component before the current registry-key/count limitation is removed from the README.
