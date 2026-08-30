# Integration API

## Stable public read model

`api.json` is refreshed after every successful state mutation. Consumers should read it as a snapshot and tolerate atomic replacement.

```json
{
  "schema_version": 3,
  "player_faction": {
    "player-uuid": "faction_id"
  },
  "factions": {
    "faction_id": {
      "name": "Faction Name",
      "members": { "player-uuid": "leader" },
      "power": 10,
      "bank": 1000
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

## External-process boundary

IPC is same-host and same-server. External services and older Pumpkin hosts should consume `api.json`; multi-server database coordination remains roadmap work.

Do not read `state.json`; its full schema is private and may migrate independently.
