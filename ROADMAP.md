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

## v0.4 — Physical cores and strategic territory

### Core establishment and membership gate

- [ ] Turn `/faction setcore` into an atomic physical-core operation that validates the location, places a `minecraft:beacon`, persists its exact world/block position, and grants the initial territory only after every step succeeds.
- [ ] Require the core chunk and every initial claim to be loaded, claimable, inside configured borders, outside protected zones, and free of conflicts with another faction before committing the core.
- [ ] Give newly created factions an `AwaitingCore` state. Until a core is successfully established, the founder may configure or disband the faction and run `/faction setcore`, but the faction cannot invite players, accept applications, accept invitations, use public joining, claim additional chunks, or receive new members through any alternate path.
- [ ] Enforce the no-core membership invariant in the domain layer as well as command/UI handlers so API consumers and future interfaces cannot bypass it.
- [ ] Define an explicit migration path for pre-v0.4 factions: preserve their members and data, require the leader to establish a physical core before further recruitment or expansion, and never create a beacon or claims silently during data loading.

### Core clearance and event-driven protection

- [ ] Validate the complete configured clearance volume once during core placement, while exempting only the approved support block or beacon-pyramid footprint below the core.
- [ ] Use cancellable block-place events as the authoritative anti-burial mechanism. Cancel prohibited placement before it appears; never place and subsequently destroy the offending block.
- [ ] Protect the core and its clearance through block-break, explosion, piston, fluid, entity-grief, and other available world-mutation events. Mark the affected core dirty when an event cannot be handled conclusively.
- [ ] Maintain a world/chunk spatial index for nearby-core and territory lookups so ordinary block events do not scan every faction.
- [ ] Add one bounded, low-frequency reconciliation task as a repair mechanism only. It must process dirty/eligible cores in small batches, skip unloaded or unavailable chunks without loading them, and treat failed or ambiguous reads as `Unknown`, never as destruction or core damage.
- [ ] Reconcile loaded cores near relevant player activity and restore an unexpectedly missing beacon without deducting lives. Scheduler observations, commands, world-edit changes, plugin errors, and chunk availability must never count as attacks.

### Core strength and destruction

- [ ] Start a level-one core with 10 configurable lives and scale maximum lives/defensive strength with core level.
- [ ] Treat a valid enemy beacon break attempt as one core hit: cancel the physical break, atomically persist one life of damage, retain/restore the beacon, provide audiovisual and faction-alert feedback, and apply a configurable global hit cooldown so tool speed or packet spam cannot remove multiple lives instantly.
- [ ] Permit core damage only through a verified gameplay event and defined war/hostility rules. Friendly, administrative, scheduler, recovery, and ambiguous actions must not deduct lives.
- [ ] At zero lives, atomically mark the core `Destroyed`, logically deactivate all territory protection, remove the physical core, and revoke all faction-owned chunks without disbanding the faction.
- [ ] Preserve faction identity, members, ranks, relations, history, and configurable bank state after core destruction. Block recruitment and expansion again until a replacement core is established under cooldown/cost rules.
- [ ] Remove territory through ownership records rather than world scans. Batch secondary cleanup and retain a recoverable destruction/claim snapshot for audit and administrator rollback.
- [ ] Persist every successful core hit and lifecycle transition crash-safely; repeated callbacks, restart recovery, or reconciliation must be idempotent and must not double-apply damage or teardown.

### Strategic chunk claiming

- [ ] Establish a new core with a 3x3 starting territory: the core's chunk plus one chunk in every horizontal direction, for nine chunks total.
- [ ] Make core level control both core strength and configurable total chunk capacity; leveling does not automatically claim a fixed-radius square beyond the initial 3x3.
- [ ] Allow factions to select each additional chunk individually, provided it is unclaimed and shares a full north/south/east/west edge with any existing faction-owned chunk. Diagonal contact alone is invalid.
- [ ] Validate each claim against capacity, world borders, safe/war/restricted zones, other ownership, core-proximity rules, and all existing claim restrictions before atomically updating persistence and the global `(world, chunk_x, chunk_z) -> faction_id` ownership index.
- [ ] Keep faction territory connected to its core. Reject unclaims or administrative mutations that would strand disconnected islands unless an explicit administrative force operation also records and repairs the resulting state.
- [ ] Support strategic, player-directed expansion toward resources and rivals while keeping minimum enemy-core distance and anti-abuse corridor/tendril rules configurable rather than hard-coded.
- [ ] Ensure destruction, replacement, migration, overclaiming, and rollback update both faction claim sets and the ownership index consistently, with startup validation capable of detecting and quarantining conflicting records.

## Note for future session agents

`src/domain.rs` is the server-authoritative contract. Preserve atomic transitions, UUID-based identity, the `FactionLookup` trait, and `api.json` schema compatibility. Never use `block_on`, nested Tokio runtimes, or Pumpkin's scheduler from ticker/runtime callbacks. A v0.4 reconciliation scheduler may be introduced only as the bounded repair mechanism specified above; it must never become the authority for combat damage or destructive faction transitions. Timed war and POW transitions remain processed from commands and player events unless deliberately redesigned. Any new inventory persistence must round-trip every item component before the current registry-key/count limitation is removed from the README.
