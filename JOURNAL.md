# Journal

Loop state + running log. Newest log entries on top. Keep entries short.

## State
- Target: **2000** (stretch 2200). Campaign spread in `campaign.json` (two slots at the top level, one and two steps below).
- Highest won: **2300** (target 2000 and stretch 2200 both passed; now probing 2400)
- Last kept engine tag: engine-v5 (promoted to `bin/destroyer-official`)
- Freeze: no  ← set to `yes` at 17:00
- Push: yes
- Campaign scoreboard (all-time, we are white, W-D-L): SF1320 1-0-0 · SF1600 1-1-0 · SF1800 1-1-0 · SF1900 1-0-0 · SF2000 4-4-1 · SF2100 4-0-1 · SF2200 5-0-1 · SF2300 1-0-0
- Progress metric (mistakes ≥100 cp by our side per game, from evals): engine-v1 9–27 per game; engine-v2 0–2; engine-v3 0–1 (SF2000 win: 0, SF2100 win: 1)

## Timeline (2026-09-25, 6h sprint 12:00–18:00)
- 12:00–12:30 infrastructure (done), push
- 12:30–16:00 engine sprint: Plan queue below, one change per round, each A/B tested; campaign runs in the background
- 16:00–17:00 fixes driven by losses at the highest unbeaten level
- 17:00 **engine freeze** (`Freeze: yes`), stop gauntlets, raise campaign to 6–8 slots at the highest unbeaten level (+ one below), abort_hopeless off
- 17:00–18:00 verify every game is saved + replays + has analysis; presentation

## Plan (queue, top = next)
0. **Pending (rules)**: analyze 2026-09-26T093447_sf2200_loss (engine-v4, g3 allowed mate after gxh6), 2026-09-26T093436_sf2000_draw (v4, 9 mistakes, gxh5 again), then the older unanalyzed ones: 092036/092128 (SF2000 wins), 092528/092936 (SF2100 wins), 091831 (SF2100 win), 091357_sf1900_win, 123747_sf1600_draw. Six per round, highest levels first.
1. King safety in eval (MG-phase only): pawn-shield bonus in front of a castled king; penalty for open/semi-open files adjacent to the king and for enemy queen/rook/bishop lines to it with no blocker; scale by enemy material. Evidence: both losses (SF2100 h4/gxh5/hxg6 then b5/bxc6 while mated; SF2200 gxh6/g3), SF2200 win 092447 (g4, Kf3 -338), SF2000 draws (gxh5, Rxf2). Regression FENs in the reports.
2. Check extensions (capped, with a qsearch node counter) + draw detection and a recursion cap inside check-evasion quiescence + SEE pruning of losing captures in qsearch + `score mate N` + regression FENs (SF2000 draws ply 47 Ke2 / ply 113 Kf3; SF2200 win 092447 ply 41 `r3kb1r/1p1b2p1/p3p3/3p1P2/3N3q/P1N1P3/1PPQPKB1/2R2R2 w kq - 3 21` expect Kg1; SF2200 win 092850 ply 29 expect Bxf5); poll the clock every 512 nodes, safety margin 150 ms
3. Killer moves + history heuristic (branching factor is 8–10× without them; est. +60–100)
4. Null-move pruning (shakmaty has no null move: flip side via FEN round-trip or a helper; disable with only king+pawns)
5. PVS + LMR + root aspiration windows
6. Pawn structure: passed pawns for both sides (rank-scaled), isolated/doubled/backward penalties; mop-up term (king distance, only when clearly ahead) with a synthetic K+R endgame suite as the test; stopgap: order pawn pushes/captures first when eval > +200
7. Small fixes: deepen the book (branching `[5,4,4,3,3,2,2]`) and add an EPD parity test between python-chess and shakmaty; record our `info depth`/`nodes` per move in the arena JSON; emit `info pv`; contempt gated on the previous searched root score with hysteresis (enter < -350, exit > -150); cap `think_time` by `wtime`; early exit from iterative deepening on proven mate; UCI `stop`

## Best verified win
- **SF 2300**: [games/2026-09-26T093830_sf2300_win.pgn](games/2026-09-26T093830_sf2300_win.pgn) (Destroyer 0.6.0, white, 5 s/move, Apple M5 Pro)
- SF 2200: [games/2026-09-26T092327_sf2200_win.pgn](games/2026-09-26T092327_sf2200_win.pgn) (engine-v3, white, 117 plies, 5 s/move, Apple M5 Pro); also [092447](games/2026-09-26T092447_sf2200_win.pgn) (v3), [092739](games/2026-09-26T092739_sf2200_win.pgn) (v4), [092850](games/2026-09-26T092850_sf2200_win.pgn) (v4)
- SF 2100: [games/2026-09-26T091234_sf2100_win.pgn](games/2026-09-26T091234_sf2100_win.pgn) (engine-v3, white, checkmate in 28 moves, 5 s/move, Apple M5 Pro)
- SF 2000: [games/2026-09-26T091300_sf2000_win.pgn](games/2026-09-26T091300_sf2000_win.pgn) (engine-v3, checkmate in 30) and [games/2026-09-25T124442_sf2000_win.pgn](games/2026-09-25T124442_sf2000_win.pgn) (engine-v2, white, checkmate in 42 moves, 5 s/move, Apple M5 Pro)
- SF 1800: [games/2026-09-25T124045_sf1800_win.pgn](games/2026-09-25T124045_sf1800_win.pgn) (engine-v2, checkmate in 53)
- SF 1320: [games/2026-09-25T114056_sf1320_win.pgn](games/2026-09-25T114056_sf1320_win.pgn) (engine-v0, mate in 25)

## Tried (and reverted)
_None yet._

## Log
- **09:55 round 5**: campaign: engine-v3/v4 WON SF2200 ×4, SF2100 ×2, SF2000 ×2, drew SF2000 ×2, lost SF2200 ×1 (v4, g3 allowed mate) · analyzed 6 games (12 reports: SF2100 loss, 4× SF2200 wins, SF2000 draw) · transposition table + hash move (2M entries, persists across moves, fail-soft) · gauntlet vs v4 +74 Elo [+31, +119] (+101 =40 -59) · kept → engine-v5, promoted · slots → 2300/2300/2200/2100 · next: king safety (repeating loss pattern at the top level).
- **09:35 round 4**: campaign (engine-v3): WIN SF2100 ×2, WIN SF2000, WIN SF1900, LOSS SF2100 (mated after hxg6/b5) · analyzed 8 games (16 agent reports) · opening book (531 positions, SF depth 18, embedded) · gauntlet vs v3 +10 [-34, +56] (openings bypass the book), startpos vs SF2000 @0.3 s: 0.729 vs 0.729 · kept as low-risk → engine-v4, promoted · slots → 2200/2200/2100/2000.
- **12:50 round 3 (cut short, session ended)**: campaign: engine-v1 drew 4 by threefold repetition (SF1600/1800/2000/2000), engine-v2 WON vs SF1800, SF1600 and **SF2000** (all checkmates) · repetition/50-move/insufficient material + contempt 30 cp · gauntlet vs v2 +47 Elo [+11, +85] (+72 =83 -45) · kept → engine-v3, promoted · slots → 2100/2100/2000/1900 · campaign runner stopped 12:45; analyses for the 7 new games still pending (Plan item 0).
- **12:50 round 2**: LOSS vs SF2000 (engine-v1 opened 1.a3 2.b3 … 8.h3, never developed, mated in 28) · tapered PeSTO PSTs · gauntlet vs v1 +346 Elo [+301, +403] (+153 =46 -1) · kept → engine-v2, promoted · campaign: SF2000 loss.
- **12:15** infrastructure: campaign runner, adjudication, Sonnet agents writing files, pipeline loop skill.
- **11:55 round 1**: WIN vs SF1320 (white, 25 moves; SF blundered 8...h6??) · quiescence search + MVV-LVA · gauntlet vs v0 +72 Elo [+41, +105] · kept → engine-v1.
- **10:45**: Scaffolded repo. Baseline engine v0: alpha-beta, material-only eval, capture-first move ordering.
