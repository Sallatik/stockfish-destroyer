---
name: improve-round
description: One round of the engine improvement loop. Plays an official game vs Stockfish at the ladder Elo, analyzes it, implements the top fix, A/B-tests it, then keeps or reverts it and updates the ladder. Designed to be driven by `/loop /improve-round`.
---

# One improvement round

State lives in `JOURNAL.md` under **State**: ladder Elo, next color, attempts at this Elo, last kept engine tag. Read it first. Everything below must leave `JOURNAL.md` and git consistent, because the next round may run on a different laptop or Claude account.

Before starting, check that `bin/destroyer` is built from the current `engine/` (run `scripts/build-engine.sh`). Never run gauntlets while an official game is playing, because CPU contention skews the 5s games.

## 1. Official game
`uv run arena --elo <ladder Elo> --color <next color>` (5s/move, ~10–15 min; run it in the background and wait).
It saves to `games/`. Alternate the color every round.

## 2. Analyze (required by the rules)
Run the `/analyze-game` skill on the new game. It dispatches the `grandmaster` and `engine-dev` agents and writes `games/analysis/<stem>.md`.

## 3. Ladder update
- **Win** → raise the ladder Elo: +200 while below 2000, +100 from 2000 on (max 3190). Reset attempts to 0. Update "Best verified win" in JOURNAL.md.
- **Draw/loss** → attempts += 1.

## 4. Pick ONE improvement
From the analysis action items, pick the highest-impact item that is **not** already listed under "Tried" in JOURNAL.md. If the analysis only suggests tiny eval tweaks while big standard features are missing, prefer the standard feature. Rough priority order for a classical engine:
quiescence search → piece-square tables (tapered mg/eg) → transposition table → MVV-LVA + killer/history ordering → repetition/50-move awareness when ahead → null-move pruning → LMR → PVS + aspiration windows → pawn structure / passed pawns → king safety → mobility → Texel tuning of eval weights.

## 5. Implement
- Edit `engine/src/`. Keep the UCI loop correct and the time budget strictly under 5s.
- `cargo build --release`, then check it runs: `printf 'uci\nisready\nposition startpos moves e2e4\ngo movetime 1000\nquit\n' | bin/destroyer` must print `bestmove`.
- Add `cargo test` perft or unit tests when touching move handling or search correctness.

## 6. A/B test: keep or revert
Baseline = the last kept tag from State (`scripts/build-engine.sh <tag>` → `bin/destroyer-<tag>`).
`uv run python -m arena.gauntlet bin/destroyer bin/destroyer-<tag> --openings 100 --time 0.1 --json games/analysis/<stem>.gauntlet.json`
- **KEEP**: commit engine changes (`engine: <feature> (+X Elo [lo, hi])`), `git tag engine-vN`, set it as the last kept tag.
- **INCONCLUSIVE**: rerun once with `--openings 250` (regenerate the openings with `-n 250` if there are fewer). Still inconclusive → keep only if the Elo estimate is > +10 and the change is standard/low-risk, otherwise revert.
- **REVERT**: `git checkout -- engine/` and rebuild. Add it to "Tried" so it isn't retried blindly (note why, e.g. "LMR −12 Elo: reductions too aggressive").

## 7. Record
- JOURNAL.md: update State; add one log line: `date · game result vs SF<elo> · change · gauntlet Elo [CI] · kept/reverted`.
- Commit `games/`, the analysis files and JOURNAL.md together: `round: <result> vs SF<elo>, <change> <kept|reverted>`.
- Do not push unless the user has said pushing is fine (see JOURNAL.md State → "Push").

## 8. Report
Three to five lines: the game result, what changed, the gauntlet result, the new ladder Elo. Then stop. The loop schedules the next round.
