---
name: analyze-game
description: Analyze a finished official game vs Stockfish. Runs the eval pass, then both required sub-agents (grandmaster + engine-dev) in parallel, which write their own reports; merges their action items into games/analysis/<stem>.md. Pass a PGN path, or omit to analyze every game that has no report yet.
---

# Analyze a game

Arguments: optional path to a `.pgn` in `games/`. If none: every `games/*.pgn` whose `games/analysis/<stem>.md` doesn't exist yet (the campaign runner adds games continuously).

For each game:

1. **Read** `games/<stem>.json` (Elo, result, adjudicated?, engine commit). Skim the PGN.

2. **Eval pass**: `uv run python -m arena.evalgame games/<stem>.pgn --out games/analysis/<stem>.evals.json` (skip if the file exists). Note the mistakes it prints.

3. **Both sub-agents, in parallel, one message with two Agent calls** (required by the rules). Give each: the PGN path, the JSON path, the evals path, the result, and the output path it must write:
   - `grandmaster` → `games/analysis/<stem>.grandmaster.md`
   - `engine-dev` → `games/analysis/<stem>.engine-dev.md` (also point it at `engine/src/`)
   Tell them what's already in the plan queue (JOURNAL.md → Plan) so they focus on what's *new* in this game. Several games can be analyzed at once: run all agents in a single message.

4. **Write** `games/analysis/<stem>.md` (short; the agents' files hold the detail):
   ```
   # <stem>: <WIN/LOSS/DRAW> vs Stockfish <elo>
   meta: color, plies, termination, engine tag/commit, hardware
   ## Summary: 3 bullets (why the game went the way it did)
   ## Action items: merged from both agents, deduplicated, max 5, ranked. Mark which are already queued in JOURNAL.md.
   Reports: [grandmaster](<stem>.grandmaster.md) · [engine-dev](<stem>.engine-dev.md) · [evals](<stem>.evals.json)
   ```

5. If an action item is new and important, add it to the JOURNAL.md Plan queue at the right priority. If it is a win at a new highest Elo, update "Best verified win".

Do not start implementing here. Report: result, top 3 action items, report path.
