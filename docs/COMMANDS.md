# Command reference

## Identity and membership

- `/faction` — open the edition-appropriate faction page.
- `/faction create <name> [public|private]`
- `/faction disband`
- `/faction invite <player>`
- `/faction apply <faction>`
- `/faction join <faction>` — accept a valid invitation.
- `/faction accept <player>` — accept a public-faction application.
- `/faction leave`
- `/faction kick <player>`
- `/faction role <player> <officer|veteran|member|recruit>`
- `/faction transfer <player>`
- `/faction public` or `/faction private`

Leaders control ownership, transfers, and roles. Leaders and officers manage invitations, applications, claims, diplomacy, prisons, homes, and the bank.

## Economy and territory

- `/faction bank [balance]`
- `/faction bank deposit <amount>`
- `/faction bank withdraw <amount>`
- `/faction claim`
- `/faction unclaim`
- `/faction overclaim` — take the current enemy chunk only when its owner has more claims than power.
- `/faction map` — show a compact five-by-five chunk map.
- `/faction sethome`
- `/faction home`

## Diplomacy, war, and POWs

- `/faction relation <faction> <neutral|truce|ally|enemy>`
- `/faction setprison`
- `/faction war <faction>` — send a consensual request lasting 72 hours.
- `/faction forcewar <faction>` — skip consent and begin preparation.
- `/faction waraccept` or `/faction wardecline`
- `/faction ready` — the first leader shortens preparation to five minutes; both leaders ready starts immediately.
- `/faction paypow <player>` — pay ransom from the faction bank.
- `/faction setarena` — admin-only, sets the global set-piece arena.

Attackers win by killing the defending leader within 30 minutes. If time expires, defenders win. Leader death settles reparations from the loser's available faction bank. Members killed during active war become POWs of the killer's faction for up to 24 hours unless ransomed.

## Mail and trade

- `/faction mail`
- `/faction trade <allied faction>` — open a 27-slot outgoing shipment.
- `/faction tradeinbox` — retrieve alliance deliveries.

Trade requires an ally relation. A sender is rejected unless the recipient has at least 27 free mailbox slots, preventing overflow loss.
