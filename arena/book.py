"""Generate the engine's opening book (engine/book.txt) with full-strength Stockfish.

We always play white in official games, so the book is a tree: at every white-to-move node
Stockfish's single best move (deep search); at every black-to-move node its top-N replies
(shallower multipv search), so the book covers the replies a limited-strength Stockfish is
most likely to choose. Output lines: `<epd> <uci move>` for white-to-move positions only.
The engine embeds the file at compile time (include_str!), so rebuild after regenerating.
"""

import argparse
import time
from pathlib import Path

import chess
import chess.engine

from arena.play import STOCKFISH

OUT = Path(__file__).resolve().parent.parent / "engine" / "book.txt"
# number of black replies to cover at black's 1st, 2nd, ... move
BRANCHING = [5, 4, 3, 3, 2]


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--white-depth", type=int, default=18, help="search depth for our (white) move")
    ap.add_argument("--black-depth", type=int, default=14, help="multipv depth for black's replies")
    ap.add_argument("--threads", type=int, default=4)
    ap.add_argument("--branching", type=int, nargs="+", default=BRANCHING)
    args = ap.parse_args()

    sf = chess.engine.SimpleEngine.popen_uci(str(STOCKFISH))
    sf.configure({"Threads": args.threads, "Hash": 256})
    book: dict[str, str] = {}
    t0 = time.time()

    def walk(board: chess.Board, level: int) -> None:
        # white to move: one best move
        epd = board.epd()
        if epd in book:
            return
        info = sf.analyse(board, chess.engine.Limit(depth=args.white_depth))
        move = info["pv"][0]
        book[epd] = move.uci()
        print(f"{len(book):4d} {'  ' * level}{board.fullmove_number}. {board.san(move)}  ({time.time() - t0:.0f}s)", flush=True)
        if level >= len(args.branching):
            return
        board.push(move)
        # black to move: cover the top-N replies
        n = args.branching[level]
        infos = sf.analyse(board, chess.engine.Limit(depth=args.black_depth), multipv=n)
        for reply in infos:
            if "pv" not in reply:
                continue
            board.push(reply["pv"][0])
            if not board.is_game_over():
                walk(board, level + 1)
            board.pop()
        board.pop()

    try:
        walk(chess.Board(), 0)
    finally:
        sf.quit()
    OUT.write_text("".join(f"{epd} {mv}\n" for epd, mv in sorted(book.items())))
    print(f"{len(book)} book positions -> {OUT}  ({time.time() - t0:.0f}s)")


if __name__ == "__main__":
    main()
