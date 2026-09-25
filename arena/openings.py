"""Generate a fixed, balanced opening set for A/B tests (so deterministic engines don't replay one game).

Walks 8 plies choosing randomly among Stockfish's top 3 moves, keeps positions Stockfish rates within
±50cp. Output is committed (arena/openings.epd) so every laptop tests on identical openings.
"""

import argparse
import random
from pathlib import Path

import chess
import chess.engine

from arena.play import STOCKFISH

OUT = Path(__file__).with_name("openings.epd")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("-n", type=int, default=100)
    ap.add_argument("--plies", type=int, default=8)
    ap.add_argument("--seed", type=int, default=42)
    args = ap.parse_args()

    rng = random.Random(args.seed)
    sf = chess.engine.SimpleEngine.popen_uci(str(STOCKFISH))
    seen: set[str] = set()
    try:
        while len(seen) < args.n:
            board = chess.Board()
            for _ in range(args.plies):
                infos = sf.analyse(board, chess.engine.Limit(depth=10), multipv=3)
                board.push(rng.choice(infos)["pv"][0])
            score = sf.analyse(board, chess.engine.Limit(depth=14))["score"].white().score(mate_score=10_000)
            if abs(score) <= 50 and not board.is_game_over():
                seen.add(board.epd())
    finally:
        sf.quit()
    OUT.write_text("\n".join(sorted(seen)) + "\n")
    print(f"{len(seen)} openings -> {OUT}")


if __name__ == "__main__":
    main()
