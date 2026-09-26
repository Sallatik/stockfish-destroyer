# Journal

Loop state + running log. Newest log entries on top. Keep entries short.

## State
- Target: **2000** (stretch 2200), both passed. Campaign spread in `campaign.json` (two slots at the top level, one and two steps below), capped at **3190** = Stockfish's max.
- **Addendum 2026-09-26**: at 3190 the score becomes the win ratio (wins / games at 3190) and the average game length in moves; both shown in the GUI ladder panel and by `uv run python -m arena.stats`.
- Level-3190 metrics: not reached yet (highest won 2800).
- Highest won: **2800** (target 2000 and stretch 2200 both passed; now probing 2900)
- Last kept engine tag: engine-v7 (promoted to `bin/destroyer-official`)
- Freeze: **yes** (11:45, demo at 12:00; engine-v7 is the frozen build). Loop stopped; campaign keeps playing 2900/2900/2800/2800 with `bin/destroyer-official` = engine-v7.
- Push: yes
- Campaign scoreboard (all-time, we are white, W-D-L): SF1320 1-0-0 · SF1600 1-1-0 · SF1800 1-1-0 · SF1900 1-0-0 · SF2000 4-4-1 · SF2100 5-1-1 · SF2200 8-0-2 · SF2300 7-0-0 · SF2400 8-0-1 · SF2500 6-0-1 · SF2600 7-2-1 · SF2700 4-0-2 · SF2800 1-4-2
- Progress metric (mistakes ≥100 cp by our side per game, from evals): engine-v1 9–27 per game; engine-v2 0–2; engine-v3 0–1 (SF2000 win: 0, SF2100 win: 1)

## Timeline (2026-09-25, 6h sprint 12:00–18:00)
- 12:00–12:30 infrastructure (done), push
- 12:30–16:00 engine sprint: Plan queue below, one change per round, each A/B tested; campaign runs in the background
- 16:00–17:00 fixes driven by losses at the highest unbeaten level
- 17:00 **engine freeze** (`Freeze: yes`), stop gauntlets, raise campaign to 6–8 slots at the highest unbeaten level (+ one below), abort_hopeless off
- 17:00–18:00 verify every game is saved + replays + has analysis; presentation

## Plan (queue, top = next)
0. **Pending (rules)**: analyze (highest levels first, six per round) 2026-09-26T112436_sf2800_win (v7, 0 mistakes, first 2800 win), 111826_sf2800_loss, 111703/112330 sf2800 draws, 112932_sf2700_loss (v7, g3 -358), 105234_sf2700_loss, 111613/110327/110930 sf2700 wins, 112841_sf2600_loss (v7, 200 plies, 16 mistakes: king walks), 105323/104248 sf2600 draws (251/337 plies), 105934/110645/111346 sf2600 wins, 104232_sf2500_win, 102421_sf2400_win, then the older backlog (SF2400 and below). Evals exist for all.
1. Check extensions (capped, with a qsearch node counter; missed forcing moves in 3 games this round: Ra6+/Kf1 at 2700, Rh7+ `r2b3R/1p3k2/6p1/8/P1P1RP2/1PB5/6PP/6K1 w - - 3 43`, Nxf7 `4k2r/p1q2pbp/b1Prpnp1/2p1N3/8/2N2Q2/PPP2PPP/R3R1K1 w k - 0 14`) + draw detection and a recursion cap inside check-evasion quiescence + SEE pruning of losing captures in qsearch + `score mate N` + regression FENs (SF2000 draws ply 47 Ke2 / ply 113 Kf3; SF2200 win 092447 ply 41 `r3kb1r/1p1b2p1/p3p3/3p1P2/3N3q/P1N1P3/1PPQPKB1/2R2R2 w kq - 3 21` expect Kg1; SF2200 win 092850 ply 29 expect Bxf5); poll the clock every 512 nodes, safety margin 150 ms
2. King safety v2 (see Tried): pawn shield + open files next to the king + enemy rook/queen already on an own-pawnless file adjacent to the king (~20/35 cp) + king escape squares; no squared attack-unit term (a linear one capped at ~60 cp at most), MG-phase only. Gate on the SF2200-loss FEN `4n1k1/6r1/3p4/QPnPp3/P1P1Pp2/2N2N1q/5PP1/3RR1K1 w - - 0 36` (reject g2g3). Evidence: both losses (SF2100 h4/gxh5/hxg6 then b5/bxc6 while mated; SF2200 gxh6/g3), SF2200 win 092447 (g4, Kf3 -338), SF2000 draws (gxh5, Rxf2). Regression FENs in the reports; v1 fixed only hxg6.
3. PVS + LMR + root aspiration windows
4. Endgame eval v2 (see Tried): EG-only passed-pawn bonus for both sides + mop-up when clearly ahead, small MG-tapered isolated/doubled penalties, bishop-pair / minor-piece imbalance term (SF2800 draw 110119: our eval +76 vs Stockfish -300 in a 2N-vs-2B ending), no rook/tropism terms at first; regression FENs as before; gauntlet at 0.1 s and 0.2 s
5. Small fixes: `arena.stats`: add "plies to convert once eval > +200" (conversion speed, from the evals files); `arena/evalgame.py`: don't flag a mistake when the played move equals the best move (false positive at ply 111 of 095553); mobility term (legal-move-count difference) folded into the pawn-structure eval pass; TT replacement policy (Exact overrides only when depth+2 ≥ stored depth), table 2^23 entries, probe/store in quiescence (from the SF2300 win review, est. +15–35); deepen the book (branching `[5,4,4,3,3,2,2]`) and add an EPD parity test between python-chess and shakmaty; record our `info depth`/`nodes` per move in the arena JSON; emit `info pv`; contempt gated on the previous searched root score with hysteresis (enter < -350, exit > -150) and scaled with it (clamp(score,0,400)/10, min 10; draws count fully against us at the top); cap `think_time` by `wtime`; early exit from iterative deepening on proven mate; UCI `stop`

## Best verified win
- **SF 2800**: [games/2026-09-26T112436_sf2800_win.pgn](games/2026-09-26T112436_sf2800_win.pgn) (Destroyer 0.8.0 = engine-v7, white, 83 plies, 5 s/move, Apple M5 Pro)
- SF 2700: [games/2026-09-26T103949_sf2700_win.pgn](games/2026-09-26T103949_sf2700_win.pgn) (Destroyer 0.7.0, white, 157 plies, 5 s/move, Apple M5 Pro)
- SF 2600: [games/2026-09-26T102023_sf2600_win.pgn](games/2026-09-26T102023_sf2600_win.pgn) (Destroyer 0.6.0, white, 75 plies, 5 s/move, Apple M5 Pro)
- SF 2500: [games/2026-09-26T101101_sf2500_win.pgn](games/2026-09-26T101101_sf2500_win.pgn) (Destroyer 0.6.0 = engine-v5, white, 121 plies, checkmate, 5 s/move, Apple M5 Pro)
- SF 2400: [games/2026-09-26T095458_sf2400_win.pgn](games/2026-09-26T095458_sf2400_win.pgn) (Destroyer 0.6.0, white, 5 s/move, Apple M5 Pro); also [095553](games/2026-09-26T095553_sf2400_win.pgn) (Destroyer 0.6.0)
- SF 2300: [games/2026-09-26T093830_sf2300_win.pgn](games/2026-09-26T093830_sf2300_win.pgn) (Destroyer 0.6.0, white, 5 s/move, Apple M5 Pro)
- SF 2200: [games/2026-09-26T092327_sf2200_win.pgn](games/2026-09-26T092327_sf2200_win.pgn) (engine-v3, white, 117 plies, 5 s/move, Apple M5 Pro); also [092447](games/2026-09-26T092447_sf2200_win.pgn) (v3), [092739](games/2026-09-26T092739_sf2200_win.pgn) (v4), [092850](games/2026-09-26T092850_sf2200_win.pgn) (v4)
- SF 2100: [games/2026-09-26T091234_sf2100_win.pgn](games/2026-09-26T091234_sf2100_win.pgn) (engine-v3, white, checkmate in 28 moves, 5 s/move, Apple M5 Pro)
- SF 2000: [games/2026-09-26T091300_sf2000_win.pgn](games/2026-09-26T091300_sf2000_win.pgn) (engine-v3, checkmate in 30) and [games/2026-09-25T124442_sf2000_win.pgn](games/2026-09-25T124442_sf2000_win.pgn) (engine-v2, white, checkmate in 42 moves, 5 s/move, Apple M5 Pro)
- SF 1800: [games/2026-09-25T124045_sf1800_win.pgn](games/2026-09-25T124045_sf1800_win.pgn) (engine-v2, checkmate in 53)
- SF 1320: [games/2026-09-25T114056_sf1320_win.pgn](games/2026-09-25T114056_sf1320_win.pgn) (engine-v0, mate in 25)

## Tried (and reverted)
- **Endgame eval pass v1** (2026-09-26 11:35, on engine-v7): passed pawns (MG 4…90 / EG 10…180 by rank, +12 connected), isolated (-12/-18), doubled (-8/-20), rook open/semi-open file (+22/+10, +10/+6), king tropism to passers (5 cp/square, EG), mop-up when ≥400 cp ahead in a phase ≤ 8 endgame. Gauntlet vs v7 at 0.1 s: **-21 [-68, +25]** (200 games), rerun **-22 [-51, +6]** (500 games, 250 openings) → reverted. It did fix the SF2700 ply-83 position (Kf1 over Ra6+). Likely double-counting PeSTO's pawn tables and over-penalising structure at shallow depth. Retry as v2: EG-only passed-pawn bonus + mop-up, small structure penalties, and test at 0.2 s as well as 0.1 s.
- **King safety v1** (2026-09-26 09:50, on engine-v5): MG-only term = pawn shield (+14 near / +7 far per file), open files next to the king (-18 own, -12 both), enemy pieces attacking the king zone as attack units (N2 B2 R3 Q5, halved without a queen) squared (up to -225). Gauntlet vs v5 at 0.1 s: **-56 Elo [-102, -12]**, +68 =32 -100 → reverted. Fixed hxg6 in the SF2100 loss position but not the deeper g3/Kf3/Rxf2 blunders. Likely too strong: the squared attack term outweighs material at shallow depth. Retry only as v2: shield + open files only, or a linear attack term capped at ~60 cp, and test at 0.2 s as well.

## Log
- **11:45 freeze**: demo in 15 min. Engine frozen at engine-v7 (Destroyer 0.8.0). Loop stopped, no gauntlets. Campaign slots 2900/2900/2800/2800. Best verified win: SF2800. Analysis backlog (agent reports) remains for ~30 lower-level games; evals exist for all.
- **11:40 round 9**: campaign: first **SF2800 win** (v7, 83 plies, 0 mistakes), SF2800 0-4-2 otherwise (draws by repetition, one loss after g4/gxf5), lost SF2700 and SF2600 once each · endgame eval pass v1 · gauntlet vs v7 -21 [-68, +25] then -22 [-51, +6] over 500 games · **reverted** (see Tried) · analyzed the 3 first SF2800 games · slots → 2900/2900/2800/2700 · openings.epd grown to 250 · next: check extensions, then king safety v2 (the SF2800 loss was pure self-inflicted king exposure).
- **10:55 round 8**: campaign: WON **SF2700** (v6, 157 plies), SF2600 ×3, SF2500 ×4, drew SF2600, lost SF2500 (v5, slow slide, no blunder) · null-move pruning (R=2, R=3 above depth 6; skipped in check, after a null, in pawn endgames) · +2 plies at 3 s on test positions · gauntlet vs v6 +58 Elo [+15, +102] (+96 =41 -63) · kept → engine-v7, promoted · slots → 2800/2800/2700/2600 · analyzing 6 games.
- **10:25 round 7**: campaign: WON **SF2600**, SF2500 ×2, SF2400 ×5, lost SF2400 ×1 and SF2200 ×1 (190 plies) · killer moves + history heuristic · gauntlet vs v5 +53 Elo [+10, +97] (+95 =40 -65) · kept → engine-v6, promoted · slots → 2700/2700/2600/2500 · analyzing 6 games (SF2500 win, SF2400 loss, SF2200 loss, 3× SF2400 wins).
- **10:00 round 6**: campaign: WON **SF2400 ×2** (v5), SF2300 ×3, SF2200 ×2, SF2100 ×1, drew SF2100 (v4, 153 plies) · slots → 2500/2500/2400/2300 · analyzed 6 games (SF2200 loss, SF2000 draw ×2, SF2300 win, SF2200 win, SF2100 wins ×2) · king safety v1 (shield/open files/attack units²) · gauntlet vs v5 **-56 Elo [-102, -12]** (+68 =32 -100) · **reverted** (see Tried) · next: check extensions + qsearch fixes.
- **09:55 round 5**: campaign: engine-v3/v4 WON SF2200 ×4, SF2100 ×2, SF2000 ×2, drew SF2000 ×2, lost SF2200 ×1 (v4, g3 allowed mate) · analyzed 6 games (12 reports: SF2100 loss, 4× SF2200 wins, SF2000 draw) · transposition table + hash move (2M entries, persists across moves, fail-soft) · gauntlet vs v4 +74 Elo [+31, +119] (+101 =40 -59) · kept → engine-v5, promoted · slots → 2300/2300/2200/2100 · next: king safety (repeating loss pattern at the top level).
- **09:35 round 4**: campaign (engine-v3): WIN SF2100 ×2, WIN SF2000, WIN SF1900, LOSS SF2100 (mated after hxg6/b5) · analyzed 8 games (16 agent reports) · opening book (531 positions, SF depth 18, embedded) · gauntlet vs v3 +10 [-34, +56] (openings bypass the book), startpos vs SF2000 @0.3 s: 0.729 vs 0.729 · kept as low-risk → engine-v4, promoted · slots → 2200/2200/2100/2000.
- **12:50 round 3 (cut short, session ended)**: campaign: engine-v1 drew 4 by threefold repetition (SF1600/1800/2000/2000), engine-v2 WON vs SF1800, SF1600 and **SF2000** (all checkmates) · repetition/50-move/insufficient material + contempt 30 cp · gauntlet vs v2 +47 Elo [+11, +85] (+72 =83 -45) · kept → engine-v3, promoted · slots → 2100/2100/2000/1900 · campaign runner stopped 12:45; analyses for the 7 new games still pending (Plan item 0).
- **12:50 round 2**: LOSS vs SF2000 (engine-v1 opened 1.a3 2.b3 … 8.h3, never developed, mated in 28) · tapered PeSTO PSTs · gauntlet vs v1 +346 Elo [+301, +403] (+153 =46 -1) · kept → engine-v2, promoted · campaign: SF2000 loss.
- **12:15** infrastructure: campaign runner, adjudication, Sonnet agents writing files, pipeline loop skill.
- **11:55 round 1**: WIN vs SF1320 (white, 25 moves; SF blundered 8...h6??) · quiescence search + MVV-LVA · gauntlet vs v0 +72 Elo [+41, +105] · kept → engine-v1.
- **10:45**: Scaffolded repo. Baseline engine v0: alpha-beta, material-only eval, capture-first move ordering.
