# Integration API

## Stable public read model

`api.json` is refreshed after every successful state mutation. Consumers should read it as a snapshot and tolerate atomic replacement.

```json
{
  "schema_version": 2,
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

## WASM limitation

Pumpkin API `0.1` does not provide a host-backed cross-plugin service registry. A separately compiled WASM plugin cannot safely borrow this plugin's in-memory Rust objects. Until the host adds that facility, integrations should use the versioned read model through an administrator-approved bridge or share this crate's domain types when compiled into a coordinated plugin.

Do not read `state.json`; its full schema is private and may migrate independently.
