# Stockfish Destroyer

A chess engine built in two days to beat Stockfish 17.1 at the highest possible `UCI_Elo`, with an always-on arena, a replay UI and an automated play → analyze → improve loop.

## Result

**Best verified win: Stockfish at Elo 2800** (white, 5 s per move, checkmate in 42 moves): [games/2026-09-26T112436_sf2800_win.pgn](games/2026-09-26T112436_sf2800_win.pgn). Sprint target was 2000, stretch 2200.

Ladder (all official games, we are always white; `uv run python -m arena.stats`):

| Elo  | W-D-L  | win ratio | avg moves |
|------|--------|-----------|-----------|
| 2900 | 0-1-0  | 0%        | 42.5      |
| 2800 | 1-5-3  | 11%       | 65.1      |
| 2700 | 4-0-2  | 67%       | 65.7      |
| 2600 | 8-2-1  | 73%       | 69.3      |
| 2500 | 6-0-1  | 86%       | 51.9      |
| 2400 | 8-0-1  | 89%       | 46.0      |
| 2300 | 7-0-0  | 100%      | 39.5      |

Stockfish's maximum is 3190; at that level the score becomes the win ratio and the average game length, both shown in the replay UI's ladder panel.

## The engine (`engine/`, Rust, ~600 lines)

Every feature was A/B tested against the previous build (200–500 games at 0.1 s/move) and kept only if it gained Elo:

| Version | Feature | Gauntlet |
|---------|---------|----------|
| v1 | quiescence search + MVV-LVA ordering | +72 |
| v2 | tapered PeSTO piece-square tables | +346 |
| v3 | repetition / 50-move detection + contempt | +47 |
| v4 | opening book (531 positions from full-strength Stockfish) | ±0 |
| v5 | transposition table + hash move | +74 |
| v6 | killer moves + history heuristic | +53 |
| v7 | null-move pruning | +58 |

Reverted: king safety v1 (-56), endgame eval pass v1 (-22). Details in [JOURNAL.md](JOURNAL.md).

## The loop

`/loop /improve-round` runs one round: ingest campaign games → analyze each one with the `grandmaster` and `engine-dev` sub-agents (reports in `games/analysis/`) → implement the next queued improvement → gauntlet → keep or revert → promote → push. The campaign runner (`uv run python -m arena.campaign`) plays official games in parallel slots around the clock and raises the level after every win.

## Run it

```bash
./scripts/setup.sh                      # pins Stockfish 17.1, builds the engine
uv run python -m arena.campaign         # official games, saved to games/
cd web && npm run dev                   # replay UI with the ladder panel
```

Rules and the 3190 addendum: [mission.md](mission.md). Team conventions: [CLAUDE.md](CLAUDE.md).
