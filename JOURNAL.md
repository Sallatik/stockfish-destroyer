# Journal

Running log of engine versions, results, and next steps. Newest on top. Keep entries short.

## State
- Ladder Elo: 1520
- Next color: black
- Attempts at this Elo: 0
- Last kept engine tag: engine-v1
- Push: no (commits stay local until the user says otherwise)

## Best verified win
- **SF 1320**: [games/2026-09-25T114056_sf1320_win.pgn](games/2026-09-25T114056_sf1320_win.pgn) (engine-v0, white, mate in 25)

## Tried (and reverted)
_None yet._

## Log
- **2026-09-25 round 1**: WIN vs SF1320 (white, 25 moves; SF blundered 8...h6??) · quiescence search + MVV-LVA · gauntlet vs v0 +72 Elo [+41, +105] · kept → engine-v1. Next: tapered PSTs, then repetition/50-move detection.
- **2026-09-25**: Scaffolded repo. Baseline engine v0: alpha-beta, material-only eval, capture-first move ordering.
