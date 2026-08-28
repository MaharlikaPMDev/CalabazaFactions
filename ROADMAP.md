# CalabazaFactions Roadmap

## v0.2 — Playable factions release

- [x] Rename plugin and repository to `CalabazaFactions`.
- [x] Creation, deletion, invitations, applications, and public/private factions.
- [x] Leader, officer, veteran, member, and recruit identity roles.
- [x] Faction bank, player wallet, power, claims, protection, and enemy overclaims.
- [x] Neutral, truce, ally, and enemy diplomacy with friendly-fire protection.
- [x] Faction Mail for applications, invitations, diplomacy, and wars.
- [x] Consensual and forced wars, 72-hour requests, preparation, readiness, arena teleport, and 30-minute battles.
- [x] POW capture, ransom, 24-hour release, and prison-boundary enforcement.
- [x] Power/troop-based war reparations.
- [x] Alliance-only item trade mailbox with capacity protection.
- [x] Java inventory faction page and Bedrock Forms page.
- [x] Atomic JSON persistence, backup, audit log, and public API snapshot.
- [x] Event-driven implementation without a repeating scheduler or Tokio blocking.

## v0.3 — Production hardening

- [ ] Persist full item data components for trade mail, including enchantments, names, lore, and custom data.
- [ ] Add configurable multi-arena selection and spawn groups per faction side.
- [ ] Add explicit war shield/cooldown and post-war grace-period state.
- [ ] Add safe-zone and war-zone administration independent of faction claims.
- [ ] Add container, piston, explosion, fluid, and entity-grief protection coverage.
- [ ] Add faction cores, upgrade trees, banners, and configurable rank permission matrices.
- [ ] Add localized messages, scoreboards, richer map overlays, and GUI actions for every command.
- [ ] Add SQLite/Postgres adapters and multi-server coordination.
- [ ] Add load tests, crash-recovery fixtures, and live Pumpkin integration tests.
- [ ] Adopt a host-backed inter-plugin service API when Pumpkin exposes one.

## Note for future session agents

`src/domain.rs` is the server-authoritative contract. Preserve atomic transitions, UUID-based identity, the `FactionLookup` trait, and `api.json` schema compatibility. Never use `block_on`, nested Tokio runtimes, or Pumpkin's scheduler from ticker/runtime callbacks. Timed war and POW transitions are intentionally processed from commands and player events. Any new inventory persistence must round-trip every item component before the current registry-key/count limitation is removed from the README.
