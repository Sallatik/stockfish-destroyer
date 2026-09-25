"""A/B test: engine A (candidate) vs engine B (baseline) over the fixed opening set, colors swapped.

Decides whether a change is kept. Fast time control (default 0.1s/move) and parallel games.
Games here are test games: they are NOT saved to games/ and NOT analyzed by agents.
"""

import argparse
import json
import math
import os
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import chess
import chess.engine

ROOT = Path(__file__).resolve().parent.parent
OPENINGS = Path(__file__).with_name("openings.epd")
MAX_PLIES = 300


def play_one(a: Path, b: Path, epd: str, a_white: bool, move_time: float) -> float:
    """Return A's score: 1 win, 0.5 draw, 0 loss."""
    board, _ = chess.Board.from_epd(epd)
    ea = chess.engine.SimpleEngine.popen_uci(str(a))
    eb = chess.engine.SimpleEngine.popen_uci(str(b))
    try:
        limit = chess.engine.Limit(time=move_time)
        while not board.is_game_over(claim_draw=True) and board.ply() < MAX_PLIES:
            a_to_move = (board.turn == chess.WHITE) == a_white
            board.push((ea if a_to_move else eb).play(board, limit).move)
    finally:
        ea.quit()
        eb.quit()
    result = board.result(claim_draw=True)
    if result == "1/2-1/2" or result == "*":
        return 0.5
    white_won = result == "1-0"
    return 1.0 if white_won == a_white else 0.0


def elo(score: float) -> float:
    score = min(max(score, 1e-6), 1 - 1e-6)
    return -400 * math.log10(1 / score - 1)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("a", type=Path, help="candidate engine")
    ap.add_argument("b", type=Path, help="baseline engine")
    ap.add_argument("--openings", type=int, default=100, help="openings to use (x2 games)")
    ap.add_argument("--time", type=float, default=0.1)
    ap.add_argument("--concurrency", type=int, default=max(1, (os.cpu_count() or 2) // 2 - 1))
    ap.add_argument("--json", type=Path, help="write summary here")
    args = ap.parse_args()

    epds = OPENINGS.read_text().split("\n")
    epds = [e for e in epds if e][: args.openings]
    jobs = [(e, w) for e in epds for w in (True, False)]

    scores: list[float] = []
    with ThreadPoolExecutor(args.concurrency) as pool:
        futs = [pool.submit(play_one, args.a, args.b, e, w, args.time) for e, w in jobs]
        for f in as_completed(futs):
            scores.append(f.result())
            if len(scores) % 50 == 0:
                print(f"  {len(scores)}/{len(jobs)} games, score {sum(scores) / len(scores):.3f}", flush=True)

    n = len(scores)
    w, d, l = scores.count(1.0), scores.count(0.5), scores.count(0.0)
    mean = sum(scores) / n
    sd = math.sqrt(sum((s - mean) ** 2 for s in scores) / n) / math.sqrt(n)
    lo, hi = elo(mean - 1.96 * sd), elo(mean + 1.96 * sd)
    los = 0.5 * (1 + math.erf((w - l) / math.sqrt(2 * (w + l)))) if w + l else 0.5
    verdict = "KEEP" if lo > 0 else ("REVERT" if hi < 5 else "INCONCLUSIVE")

    summary = {
        "a": str(args.a), "b": str(args.b), "games": n, "wins": w, "draws": d, "losses": l,
        "score": round(mean, 4), "elo": round(elo(mean), 1), "elo_95": [round(lo, 1), round(hi, 1)],
        "los": round(los, 4), "move_time_s": args.time, "verdict": verdict,
    }
    print(f"A vs B: +{w} ={d} -{l}  score {mean:.3f}  Elo {elo(mean):+.0f} [{lo:+.0f}, {hi:+.0f}]  LOS {los:.1%}  -> {verdict}")
    if args.json:
        args.json.write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
