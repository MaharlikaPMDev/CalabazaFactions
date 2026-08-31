# Integration API

## Stable public read model

`api.json` is refreshed after every successful state mutation. Consumers should read it as a snapshot and tolerate atomic replacement.

```json
{
  "schema_version": 4,
  "player_faction": {
    "player-uuid": "faction_id"
  },
  "factions": {
    "faction_id": {
      "name": "Faction Name",
      "members": { "player-uuid": "leader" },
      "power": 10,
      "bank": 1000,
      "core_lifecycle": "active",
      "physical_core": {
        "location": {"world":"world","x":8,"y":64,"z":8},
        "lives": 10,
        "max_lives": 10
      },
      "claims": []
    }
  }
}
```

Use UUID strings as player keys. Display names are not stable identifiers.

## Rust contract

The public `domain` module exports `FactionState`, `Faction`, `Role`, `Relation`, and `FactionLookup`. `FactionLookup` provides:

- `faction_id(player_uuid)`
- `faction_name(player_uuid)`
- `relation_between_players(first_uuid, second_uuid)`

## Host IPC contract

Pumpkin plugins can send UTF-8 JSON to the `CalabazaFactions` plugin ID. Responses are UTF-8 JSON and currently use IPC contract version 1.

Faction lookup:

```json
{"action":"faction","player":"player-uuid"}
```

```json
{"version":1,"faction_id":"faction_id","faction_name":"Faction Name"}
```

Relation lookup:

```json
{"action":"relation","first":"first-player-uuid","second":"second-player-uuid"}
```

```json
{"version":1,"relation":"ally"}
```

Unknown players return `null` faction fields and `neutral` relations. Consumers must ignore unknown response fields so the contract can grow compatibly.

Capabilities:

```json
{"action":"capabilities"}
```

The response identifies IPC schema version 1, supported actions, event schema, and topic families.

## Event subscriptions

The subscribing plugin's host-provided ID is authoritative; callers cannot subscribe on behalf of another plugin.

```json
{"action":"subscribe","topics":["core.*","territory.*","war.*"]}
```

Use `{"action":"unsubscribe"}` to remove all live topics. Exact event names, a family ending in `.*`, and `*` are accepted filters. Current events include:

- `faction.created`, `faction.disbanded`
- `core.established`, `core.attacked`, `core.destroyed`, `core.restored`
- `territory.claimed`, `territory.overclaimed`, `territory.unclaimed`
- `war.declared`, `war.accepted`, `war.declined`, `war.started`, `war.ended`

`raid.*` is reserved until CalabazaFactions has an authoritative raid lifecycle.

Every live notification and journal record uses this envelope:

```json
{
  "schema": "calabazafactions.event",
  "version": 1,
  "sequence": 42,
  "event_type": "core.attacked",
  "timestamp": 1788148800,
  "data": {
    "faction_id": "knights",
    "attacker_uuid": "player-uuid",
    "remaining_lives": 9
  }
}
```

Recover missed messages with:

```json
{"action":"events_since","since":41,"topics":["core.*"]}
```

The response includes `oldest_sequence`, `latest_sequence`, `gap`, and `events`. If `gap` is true, the requested cursor predates retained history and the listener should reconcile from current lookup/snapshot state before advancing. Sequence numbers are monotonic and never reused. Gameplay commits do not wait for listeners; consumers must deduplicate by sequence and use journal recovery after reloads.

## CalabazaBank economy contract

When `[economy].mode = "external"`, CalabazaFactions sends UTF-8 JSON to the configured provider ID (`CalabazaBank` by default). The contract uses schema `calabazabank.ipc`, version 1, and these actions:

```json
{"schema":"calabazabank.ipc","version":1,"action":"health"}
{"schema":"calabazabank.ipc","version":1,"action":"balance","account_id":"player-uuid"}
{"schema":"calabazabank.ipc","version":1,"action":"debit","account_id":"player-uuid","amount":100,"transaction_id":"unique-id","reason":"faction_bank_deposit"}
{"schema":"calabazabank.ipc","version":1,"action":"credit","account_id":"player-uuid","amount":100,"transaction_id":"unique-id","reason":"faction_bank_withdrawal"}
```

Successful balance responses contain `{"ok":true,"balance":1000}`. Mutation responses contain `{"ok":true,"balance":900}`. Failures contain `{"ok":false,"error":"reason"}`. CalabazaBank must treat `transaction_id` idempotently: retrying the same logical transaction returns its prior result and never applies it twice. CalabazaFactions remains the owner of faction-bank balances but is not the source of the global player economy in external mode.

## External-process boundary

IPC is same-host and same-server. External services and older Pumpkin hosts should consume `api.json`; multi-server database coordination remains roadmap work.

Do not read `state.json`; its full schema is private and may migrate independently.
