# CalabazaFaction design notes

## Reference mechanics

Classic Factions systems commonly combine chunk claims, faction/member power, over-claim vulnerability, and relations such as neutral, truce, ally, and enemy. Hardcore variants add scheduled raid windows, shields, deaths-till-raidable, faction cores, and auditable war outcomes.

## Design principles

1. **Server-authoritative state:** claims, power, war timers, and bank balances are written by one serialized state layer.
2. **Atomic war transitions:** starting, pausing, winning, and ending a raid must be crash-safe and idempotent.
3. **No silent loss:** every claim transfer, power loss, bank transaction, and role change is logged.
4. **Configurable pressure:** servers can choose power mode, deaths-till-raidable mode, or a hybrid.
5. **Edition parity:** Java GUIs and Bedrock forms expose the same actions, while protection remains server-side.

## Proposed domain entities

`Faction`, `Role`, `Claim`, `Relation`, `War`, `WarPolicyState`, `Arena`, `Zone`, `UpgradeKind`, `TradeItem`, and `AuditEvent`.

## Open decisions

- Whether a claim is a single 16×16 chunk or supports configurable region sizes.
- Whether raid destruction is temporary, repairable, or permanent.
- How offline members contribute power and how inactivity decay works.
- Which Pumpkin block/entity events and packet APIs are stable enough for enforcement.

## v0.3 decisions

- Named arenas rotate in deterministic sorted order and distribute online members across each side's spawn group.
- Safe zones override war zones when administrators overlap them. War zones override faction claim build/PvP policy.
- Rank permissions are configuration, while leadership ownership operations remain leader-only invariants.
- Item mail stores the pinned Pumpkin data-component enum index and opaque serialized bytes, allowing exact component restoration without interpreting component payloads.
- Same-server integrations use versioned host IPC; `api.json` remains the stable external snapshot.
- Nested GUI opens are forbidden from inventory/form response callbacks because the pinned host can re-enter its runtime there.
