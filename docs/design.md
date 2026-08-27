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

`Faction`, `MemberRole`, `Claim`, `Relation`, `PowerLedger`, `Raid`, `Shield`, `Core`, `BankAccount`, `AuditEvent`.

## Open decisions

- Whether a claim is a single 16×16 chunk or supports configurable region sizes.
- Whether raid destruction is temporary, repairable, or permanent.
- How offline members contribute power and how inactivity decay works.
- Which Pumpkin block/entity events and packet APIs are stable enough for enforcement.
