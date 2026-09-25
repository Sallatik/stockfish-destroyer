---
name: analyze-game
description: Analyze a finished game vs Stockfish. Runs both required sub-agents (grandmaster + engine-dev), writes a report to games/analysis/, and updates JOURNAL.md. Use after every game. Pass a PGN path, or default to the newest game.
---

# Analyze a game

Arguments: optional path to a `.pgn` in `games/`. If none, use the newest `games/*.pgn` by filename.

## Steps

1. **Load the game.** Read the `.pgn` and its matching `.json` (same stem). Note Stockfish Elo, our color, result, engine commit, hardware.

2. **Eval pass** (gives both agents shared facts). Run:
   `uv run python -m arena.evalgame <pgn> --out games/analysis/<stem>.evals.json`
   This writes a per-ply Stockfish eval (full strength, fixed depth) and flags our moves with large eval drops.

3. **Dispatch both sub-agents in parallel.** This is required by the rules. Send a single message with two Agent calls:
   - `grandmaster`: give it the PGN path, the metadata, and the evals file path.
   - `engine-dev`: same inputs, plus a pointer to `engine/src/`.

4. **Write the report** to `games/analysis/<stem>.md`:
   ```
   # <stem>: <WIN/LOSS/DRAW> vs Stockfish <elo>
   meta: color, plies, engine commit, hardware
   ## Summary (3 bullets, yours)
   ## Grandmaster analysis (agent output, verbatim)
   ## Engine-dev analysis (agent output, verbatim)
   ## Action items (merged, deduplicated, ranked: max 5)
   ```

5. **Update `JOURNAL.md`.** Add one log line (date, result, Elo, top action item). If this is a win at a new highest Elo, update "Best verified win" with the PGN path.

6. Tell the user the result, the top 3 action items, and the report path. Do not start implementing unless asked (or unless running inside the improvement loop).
