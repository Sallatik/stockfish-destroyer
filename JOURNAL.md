# Journal

Loop state + running log. Newest log entries on top. Keep entries short.

## State
- Target: **2000** (stretch 2200). Campaign spread in `campaign.json` (two slots at the top level, one and two steps below).
- Highest won: **2400** (target 2000 and stretch 2200 both passed; now probing 2500)
- Last kept engine tag: engine-v5 (promoted to `bin/destroyer-official`)
- Freeze: no  ← set to `yes` at 17:00
- Push: yes
- Campaign scoreboard (all-time, we are white, W-D-L): SF1320 1-0-0 · SF1600 1-1-0 · SF1800 1-1-0 · SF1900 1-0-0 · SF2000 4-4-1 · SF2100 5-1-1 · SF2200 6-0-1 · SF2300 4-0-0 · SF2400 2-0-0
- Progress metric (mistakes ≥100 cp by our side per game, from evals): engine-v1 9–27 per game; engine-v2 0–2; engine-v3 0–1 (SF2000 win: 0, SF2100 win: 1)

## Timeline (2026-09-25, 6h sprint 12:00–18:00)
- 12:00–12:30 infrastructure (done), push
- 12:30–16:00 engine sprint: Plan queue below, one change per round, each A/B tested; campaign runs in the background
- 16:00–17:00 fixes driven by losses at the highest unbeaten level
- 17:00 **engine freeze** (`Freeze: yes`), stop gauntlets, raise campaign to 6–8 slots at the highest unbeaten level (+ one below), abort_hopeless off
- 17:00–18:00 verify every game is saved + replays + has analysis; presentation

## Plan (queue, top = next)
0. **Pending (rules)**: analyze (highest levels first, six per round) 2026-09-26T095458_sf2400_win (v5, 0 mistakes), 095553_sf2400_win (v5, 121 plies, c7 at ply 111 allowed mate, SF missed it), 095339_sf2300_win (v5, Qc6+ -596), 094834_sf2300_win (v5, hxg5 -330 then Bb6/Qa3 each allowed mate), 094035_sf2300_win, 094924_sf2200_win, 094154_sf2100_draw (v4, 153 plies, Kd2/Kc2 allowed mates), 094202_sf2100_win, 092036/092128 (SF2000 wins), 091831 (SF2100 win), 091357_sf1900_win, 2026-09-25T123747_sf1600_draw. Evals exist for all.
1. Killer moves + history heuristic (branching factor is 8–10× without them; est. +60–100)
2. Check extensions (capped, with a qsearch node counter) + draw detection and a recursion cap inside check-evasion quiescence + SEE pruning of losing captures in qsearch + `score mate N` + regression FENs (SF2000 draws ply 47 Ke2 / ply 113 Kf3; SF2200 win 092447 ply 41 `r3kb1r/1p1b2p1/p3p3/3p1P2/3N3q/P1N1P3/1PPQPKB1/2R2R2 w kq - 3 21` expect Kg1; SF2200 win 092850 ply 29 expect Bxf5); poll the clock every 512 nodes, safety margin 150 ms
3. King safety v2 (see Tried): pawn shield + open files next to the king + enemy rook/queen already on an own-pawnless file adjacent to the king (~20/35 cp) + king escape squares; no squared attack-unit term (a linear one capped at ~60 cp at most), MG-phase only. Gate on the SF2200-loss FEN `4n1k1/6r1/3p4/QPnPp3/P1P1Pp2/2N2N1q/5PP1/3RR1K1 w - - 0 36` (reject g2g3). Evidence: both losses (SF2100 h4/gxh5/hxg6 then b5/bxc6 while mated; SF2200 gxh6/g3), SF2200 win 092447 (g4, Kf3 -338), SF2000 draws (gxh5, Rxf2). Regression FENs in the reports; v1 fixed only hxg6.
4. Null-move pruning (R=2; shakmaty `Position::swap_turn()` per engine-dev, verify it exists; skip when in check or with only king+pawns; several reports want it right after killer/history)
5. PVS + LMR + root aspiration windows
6. Pawn structure: passed pawns for both sides (rank-scaled), isolated/doubled/backward penalties; mop-up term (king distance, only when clearly ahead) with a synthetic K+R endgame suite as the test; stopgap: order pawn pushes/captures first when eval > +200
7. Small fixes: TT replacement policy (Exact overrides only when depth+2 ≥ stored depth), table 2^23 entries, probe/store in quiescence (from the SF2300 win review, est. +15–35); deepen the book (branching `[5,4,4,3,3,2,2]`) and add an EPD parity test between python-chess and shakmaty; record our `info depth`/`nodes` per move in the arena JSON; emit `info pv`; contempt gated on the previous searched root score with hysteresis (enter < -350, exit > -150); cap `think_time` by `wtime`; early exit from iterative deepening on proven mate; UCI `stop`

## Best verified win
- **SF 2400**: [games/2026-09-26T095458_sf2400_win.pgn](games/2026-09-26T095458_sf2400_win.pgn) (Destroyer 0.6.0, white, 5 s/move, Apple M5 Pro); also [095553](games/2026-09-26T095553_sf2400_win.pgn) (Destroyer 0.6.0)
- SF 2300: [games/2026-09-26T093830_sf2300_win.pgn](games/2026-09-26T093830_sf2300_win.pgn) (Destroyer 0.6.0, white, 5 s/move, Apple M5 Pro)
- SF 2200: [games/2026-09-26T092327_sf2200_win.pgn](games/2026-09-26T092327_sf2200_win.pgn) (engine-v3, white, 117 plies, 5 s/move, Apple M5 Pro); also [092447](games/2026-09-26T092447_sf2200_win.pgn) (v3), [092739](games/2026-09-26T092739_sf2200_win.pgn) (v4), [092850](games/2026-09-26T092850_sf2200_win.pgn) (v4)
- SF 2100: [games/2026-09-26T091234_sf2100_win.pgn](games/2026-09-26T091234_sf2100_win.pgn) (engine-v3, white, checkmate in 28 moves, 5 s/move, Apple M5 Pro)
- SF 2000: [games/2026-09-26T091300_sf2000_win.pgn](games/2026-09-26T091300_sf2000_win.pgn) (engine-v3, checkmate in 30) and [games/2026-09-25T124442_sf2000_win.pgn](games/2026-09-25T124442_sf2000_win.pgn) (engine-v2, white, checkmate in 42 moves, 5 s/move, Apple M5 Pro)
- SF 1800: [games/2026-09-25T124045_sf1800_win.pgn](games/2026-09-25T124045_sf1800_win.pgn) (engine-v2, checkmate in 53)
- SF 1320: [games/2026-09-25T114056_sf1320_win.pgn](games/2026-09-25T114056_sf1320_win.pgn) (engine-v0, mate in 25)

## Tried (and reverted)
- **King safety v1** (2026-09-26 09:50, on engine-v5): MG-only term = pawn shield (+14 near / +7 far per file), open files next to the king (-18 own, -12 both), enemy pieces attacking the king zone as attack units (N2 B2 R3 Q5, halved without a queen) squared (up to -225). Gauntlet vs v5 at 0.1 s: **-56 Elo [-102, -12]**, +68 =32 -100 → reverted. Fixed hxg6 in the SF2100 loss position but not the deeper g3/Kf3/Rxf2 blunders. Likely too strong: the squared attack term outweighs material at shallow depth. Retry only as v2: shield + open files only, or a linear attack term capped at ~60 cp, and test at 0.2 s as well.

## Log
- **10:00 round 6**: campaign: WON **SF2400 ×2** (v5), SF2300 ×3, SF2200 ×2, SF2100 ×1, drew SF2100 (v4, 153 plies) · slots → 2500/2500/2400/2300 · analyzed 6 games (SF2200 loss, SF2000 draw ×2, SF2300 win, SF2200 win, SF2100 wins ×2) · king safety v1 (shield/open files/attack units²) · gauntlet vs v5 **-56 Elo [-102, -12]** (+68 =32 -100) · **reverted** (see Tried) · next: check extensions + qsearch fixes.
- **09:55 round 5**: campaign: engine-v3/v4 WON SF2200 ×4, SF2100 ×2, SF2000 ×2, drew SF2000 ×2, lost SF2200 ×1 (v4, g3 allowed mate) · analyzed 6 games (12 reports: SF2100 loss, 4× SF2200 wins, SF2000 draw) · transposition table + hash move (2M entries, persists across moves, fail-soft) · gauntlet vs v4 +74 Elo [+31, +119] (+101 =40 -59) · kept → engine-v5, promoted · slots → 2300/2300/2200/2100 · next: king safety (repeating loss pattern at the top level).
- **09:35 round 4**: campaign (engine-v3): WIN SF2100 ×2, WIN SF2000, WIN SF1900, LOSS SF2100 (mated after hxg6/b5) · analyzed 8 games (16 agent reports) · opening book (531 positions, SF depth 18, embedded) · gauntlet vs v3 +10 [-34, +56] (openings bypass the book), startpos vs SF2000 @0.3 s: 0.729 vs 0.729 · kept as low-risk → engine-v4, promoted · slots → 2200/2200/2100/2000.
- **12:50 round 3 (cut short, session ended)**: campaign: engine-v1 drew 4 by threefold repetition (SF1600/1800/2000/2000), engine-v2 WON vs SF1800, SF1600 and **SF2000** (all checkmates) · repetition/50-move/insufficient material + contempt 30 cp · gauntlet vs v2 +47 Elo [+11, +85] (+72 =83 -45) · kept → engine-v3, promoted · slots → 2100/2100/2000/1900 · campaign runner stopped 12:45; analyses for the 7 new games still pending (Plan item 0).
- **12:50 round 2**: LOSS vs SF2000 (engine-v1 opened 1.a3 2.b3 … 8.h3, never developed, mated in 28) · tapered PeSTO PSTs · gauntlet vs v1 +346 Elo [+301, +403] (+153 =46 -1) · kept → engine-v2, promoted · campaign: SF2000 loss.
- **12:15** infrastructure: campaign runner, adjudication, Sonnet agents writing files, pipeline loop skill.
- **11:55 round 1**: WIN vs SF1320 (white, 25 moves; SF blundered 8...h6??) · quiescence search + MVV-LVA · gauntlet vs v0 +72 Elo [+41, +105] · kept → engine-v1.
- **10:45**: Scaffolded repo. Baseline engine v0: alpha-beta, material-only eval, capture-first move ordering.
