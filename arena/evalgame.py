"""Per-ply full-strength Stockfish evals for a saved game, flagging our mistakes. Used by the analyze-game skill."""

import argparse
import json
from pathlib import Path

import chess
import chess.engine
import chess.pgn

from arena.play import STOCKFISH


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("pgn", type=Path)
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--depth", type=int, default=18)
    ap.add_argument("--blunder-cp", type=int, default=100, help="eval drop that counts as a mistake")
    args = ap.parse_args()

    meta = json.loads(args.pgn.with_suffix(".json").read_text())
    our_color = chess.WHITE if meta["our_color"] == "white" else chess.BLACK
    game = chess.pgn.read_game(args.pgn.open())
    board = game.board()

    sf = chess.engine.SimpleEngine.popen_uci(str(STOCKFISH))
    limit = chess.engine.Limit(depth=args.depth)
    plies = []
    try:
        info = sf.analyse(board, limit)
        for move in game.mainline_moves():
            mover = board.turn
            before = info["score"].pov(mover).score(mate_score=10_000)
            best = info.get("pv", [None])[0]
            san = board.san(move)
            fen = board.fen()
            board.push(move)
            info = sf.analyse(board, limit)
            after = info["score"].pov(mover).score(mate_score=10_000)
            drop = before - after
            plies.append({
                "ply": len(plies) + 1,
                "fen_before": fen,
                "side": "ours" if mover == our_color else "stockfish",
                "move": san,
                "best": chess.Board(fen).san(best) if best else None,
                "eval_before_cp": before,
                "eval_after_cp": after,
                "drop_cp": drop,
                "mistake": mover == our_color and drop >= args.blunder_cp,
            })
    finally:
        sf.quit()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps({"meta": meta, "depth": args.depth, "plies": plies}, indent=2) + "\n")
    mistakes = [p for p in plies if p["mistake"]]
    print(f"{len(plies)} plies, {len(mistakes)} mistakes by our engine (>= {args.blunder_cp}cp) -> {args.out}")
    for p in mistakes:
        print(f"  ply {p['ply']}: {p['move']} (best {p['best']}), -{p['drop_cp']}cp")


if __name__ == "__main__":
    main()
