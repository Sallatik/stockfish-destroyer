---
name: improve-round
description: One round of the engine improvement loop (run via `/loop /improve-round`). Analyzes any new campaign games, updates the campaign levels, implements the next queued engine improvement, A/B-tests it, keeps or reverts it, promotes kept builds to the campaign runner, records everything in JOURNAL.md and pushes.
---

# One improvement round

All state lives in `JOURNAL.md` (**State**, **Plan** queue, **Tried**) and `campaign.json`. Read JOURNAL.md first. A round may run on a different laptop or Claude account than the previous one, so git + JOURNAL.md must be left consistent at the end.

## 0. Health checks (1 minute)
- Usage: if a usage/limits tool is available, check the 5-hour limit. Above 80%: do a *cheap round* (skip step 3's agents if no new games need it, and stop after step 5). Above 95%: only record state, then tell the user to switch accounts.
- Campaign runner: `pgrep -f arena.campaign` must find a process. If not, tell the user to start it (`uv run python -m arena.campaign` in a terminal tab) and continue anyway.
- Engine freeze: if JOURNAL.md State says `Freeze: yes`, do only steps 1–2 and 6, then stop.
- `bin/destroyer` must be built from the current `engine/` (`scripts/build-engine.sh`).

## 1. Ingest campaign results
`tail -30 campaign.log` and `git status --short games/`. For each new game, note Elo and result.
- A **win** at Elo X above the current best: update State → "Best verified win" and "Highest won". Then shift `campaign.json` slots up: every slot ≤ X becomes X+100, keep the spread shape (two at the top level, one and two steps below). **Cap: 3190** (Stockfish's maximum). Any slot that would exceed 3190 becomes 3190.
- **At the top (addendum)**: once "Highest won" is 3190, set *every* slot to 3190 and stop shifting. From then on the score is the **win ratio at 3190** (wins / all games at 3190) and the **average game length in moves**; record both in State → "Level-3190 metrics" every round and make the engine change that most improves the ratio (draws count against us, so contempt and conversion speed matter).
- Update State → "Campaign scoreboard" with `uv run python -m arena.stats` (per level: W-D-L, win ratio, average moves). The same numbers are shown in the replay UI's ladder panel; keep them consistent.

## 2. Analyze new games (required by the rules)
Run the `analyze-game` skill with no argument: it analyzes every game without a report, all agents in one parallel batch. If there are more than 6 unanalyzed games, analyze the 6 most recent at the highest levels now and the rest next round.

## 3. Pick ONE change
Take the top item of the JOURNAL.md **Plan** queue unless the new analyses reveal something that clearly costs more (a bug, a repeating blunder pattern at the top level), in which case put that first. Never pick something in **Tried** without a new reason. Standard priority for missing features: PSTs → repetition/50-move + contempt → opening book → TT + hash move → killer/history → null move → LMR → PVS/aspiration → pawn structure → king safety → mobility → eval tuning.

## 4. Implement
- Edit `engine/src/`. Keep the UCI loop correct and the per-move budget strictly under 5 s.
- `scripts/build-engine.sh`, then: `printf 'uci\nisready\nposition startpos moves e2e4\ngo movetime 1000\nquit\n' | bin/destroyer` must end with `bestmove`.
- Add `cargo test` tests when touching move handling, hashing or repetition logic.
- Bump `version` in `engine/Cargo.toml` (it shows up in the PGN headers).

## 5. A/B test → keep or revert
Baseline = "Last kept engine tag" from State (`scripts/build-engine.sh <tag>` → `bin/destroyer-<tag>`, once).
`uv run python -m arena.gauntlet bin/destroyer bin/destroyer-<tag> --openings 100 --time 0.1 --concurrency 6 --json games/analysis/gauntlet-<feature>.json`
(concurrency 6 leaves cores for the campaign's 5 s games; never raise it while the campaign runs.)
- **KEEP**: `git add engine && git commit -m "engine: <feature> (+X Elo [lo, hi])"`, `git tag engine-vN`, update State → last kept tag, then **`scripts/promote.sh`** so the campaign uses it.
- **INCONCLUSIVE**: rerun once with `--openings 250` (generate more openings with `uv run python -m arena.openings -n 250` if needed). Still inconclusive → keep if the Elo estimate is > +10 and the change is standard/low-risk, otherwise revert.
- **REVERT**: `git checkout -- engine/ && scripts/build-engine.sh`. Add to **Tried** with the number and a one-line reason.
- Features that mostly help at depth (TT, null move, LMR) can look small at 0.1 s. If INCONCLUSIVE but positive, keep them.

## 6. Record and push
- JOURNAL.md: pop the item from Plan; update State; add a Log line: `HH:MM · <change> · gauntlet +X [lo, hi] · kept/reverted · campaign: <new results this round>`.
- `git add -A games JOURNAL.md campaign.json && git commit -m "round: <change> <kept|reverted>; <n> campaign games"`
- `git push` (State → Push: yes). If the push is rejected, `git pull --rebase` and push again; games never conflict (unique filenames), `games/index.json` can be regenerated with `uv run python -c "from arena.play import rebuild_index; rebuild_index()"`.

## 7. Report and stop
Five lines max: campaign results this round, best win so far (and, once at 3190, the win ratio + average moves there), the change, gauntlet result, what's next. Then stop: the loop schedules the next round. If the Plan queue is empty and there are no new action items, say so and suggest the user set `Freeze: yes`.
