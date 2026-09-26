# Journal

Loop state + running log. Newest log entries on top. Keep entries short.

## State
- Target: **2000** (stretch 2200). Campaign spread in `campaign.json` (two slots at the top level, one and two steps below).
- Highest won: **2100** (target 2000 reached; stretch 2200 is one step away)
- Last kept engine tag: engine-v4 (promoted to `bin/destroyer-official`)
- Freeze: no  ← set to `yes` at 17:00
- Push: yes
- Campaign scoreboard (all-time, we are white, W-D-L): SF1320 1-0-0 · SF1600 1-1-0 · SF1800 1-1-0 · SF1900 1-0-0 · SF2000 2-2-1 · SF2100 2-0-1
- Progress metric (mistakes ≥100 cp by our side per game, from evals): engine-v1 9–27 per game; engine-v2 0–2; engine-v3 0–1 (SF2000 win: 0, SF2100 win: 1)

## Timeline (2026-09-25, 6h sprint 12:00–18:00)
- 12:00–12:30 infrastructure (done), push
- 12:30–16:00 engine sprint: Plan queue below, one change per round, each A/B tested; campaign runs in the background
- 16:00–17:00 fixes driven by losses at the highest unbeaten level
- 17:00 **engine freeze** (`Freeze: yes`), stop gauntlets, raise campaign to 6–8 slots at the highest unbeaten level (+ one below), abort_hopeless off
- 17:00–18:00 verify every game is saved + replays + has analysis; presentation

## Plan (queue, top = next)
1. Transposition table + hash-move ordering (also fixes the shallow-tactic misses at ply 39 of the SF1800 win and axb5 in the SF2000 win)
2. Check extensions (capped, with a qsearch node counter) + draw detection and a recursion cap inside check-evasion quiescence + SEE pruning of losing captures in qsearch + `score mate N` + regression FENs from the SF2000 draws (ply 47 Ke2, ply 113 Kf3); poll the clock every 512 nodes, safety margin 150 ms
3. King safety: penalty for open file/diagonal from enemy queen/rook to our king with no blocker, castling-rights loss, pawn shield (three SF2000 games show king walks/shuffles)
4. Killer moves + history heuristic
5. Null-move pruning
6. PVS + LMR + root aspiration windows
7. Pawn structure: passed pawns for both sides (rank-scaled), isolated/doubled/backward penalties; mop-up term
8. Small fixes: record our `info depth`/`nodes` per move in the arena JSON (first: makes every report sharper); emit `info pv`; contempt gated on the previous searched root score with hysteresis (enter < -350, exit > -150); cap `think_time` by `wtime`; early exit from iterative deepening on proven mate; UCI `stop`
0. **Pending (rules)**: analyze 2026-09-26T091357_sf2100_loss (engine-v3, mated: ply 43 hxg6 -454, ply 49 b5 allowed mate), 2026-09-26T091831_sf2100_win, 2026-09-26T091357_sf1900_win (evals exist) and the old 2026-09-25T123747_sf1600_draw. Run `/analyze-game` first next round.

## Best verified win
- **SF 2100**: [games/2026-09-26T091234_sf2100_win.pgn](games/2026-09-26T091234_sf2100_win.pgn) (engine-v3, white, checkmate in 28 moves, 5 s/move, Apple M5 Pro)
- SF 2000: [games/2026-09-26T091300_sf2000_win.pgn](games/2026-09-26T091300_sf2000_win.pgn) (engine-v3, checkmate in 30) and [games/2026-09-25T124442_sf2000_win.pgn](games/2026-09-25T124442_sf2000_win.pgn) (engine-v2, white, checkmate in 42 moves, 5 s/move, Apple M5 Pro)
- SF 1800: [games/2026-09-25T124045_sf1800_win.pgn](games/2026-09-25T124045_sf1800_win.pgn) (engine-v2, checkmate in 53)
- SF 1320: [games/2026-09-25T114056_sf1320_win.pgn](games/2026-09-25T114056_sf1320_win.pgn) (engine-v0, mate in 25)

## Tried (and reverted)
_None yet._

## Log
- **09:35 round 4**: campaign (engine-v3): WIN SF2100 ×2, WIN SF2000, WIN SF1900, LOSS SF2100 (mated after hxg6/b5) · analyzed 8 games (16 agent reports) · opening book (531 positions, SF depth 18, embedded) · gauntlet vs v3 +10 [-34, +56] (openings bypass the book), startpos vs SF2000 @0.3 s: 0.729 vs 0.729 · kept as low-risk → engine-v4, promoted · slots → 2200/2200/2100/2000.
- **12:50 round 3 (cut short, session ended)**: campaign: engine-v1 drew 4 by threefold repetition (SF1600/1800/2000/2000), engine-v2 WON vs SF1800, SF1600 and **SF2000** (all checkmates) · repetition/50-move/insufficient material + contempt 30 cp · gauntlet vs v2 +47 Elo [+11, +85] (+72 =83 -45) · kept → engine-v3, promoted · slots → 2100/2100/2000/1900 · campaign runner stopped 12:45; analyses for the 7 new games still pending (Plan item 0).
- **12:50 round 2**: LOSS vs SF2000 (engine-v1 opened 1.a3 2.b3 … 8.h3, never developed, mated in 28) · tapered PeSTO PSTs · gauntlet vs v1 +346 Elo [+301, +403] (+153 =46 -1) · kept → engine-v2, promoted · campaign: SF2000 loss.
- **12:15** infrastructure: campaign runner, adjudication, Sonnet agents writing files, pipeline loop skill.
- **11:55 round 1**: WIN vs SF1320 (white, 25 moves; SF blundered 8...h6??) · quiescence search + MVV-LVA · gauntlet vs v0 +72 Elo [+41, +105] · kept → engine-v1.
- **10:45**: Scaffolded repo. Baseline engine v0: alpha-beta, material-only eval, capture-first move ordering.
