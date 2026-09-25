"""Play our engine against Elo-limited Stockfish and save the game to games/."""

import argparse
import datetime as dt
import json
import os
import platform
import subprocess
from pathlib import Path

import chess
import chess.engine
import chess.pgn

ROOT = Path(__file__).resolve().parent.parent
GAMES = ROOT / "games"
STOCKFISH = ROOT / "bin" / "stockfish"
OUR_ENGINE = ROOT / "bin" / "destroyer"
MOVE_TIME = 5.0  # rules: max 5 seconds per move, per player


def git_commit() -> str:
    try:
        sha = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, text=True).strip()
        dirty = subprocess.call(["git", "diff", "--quiet", "--", "engine"], cwd=ROOT) != 0
        return sha + ("-dirty" if dirty else "")
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def hardware() -> str:
    if platform.system() == "Darwin":
        cpu = subprocess.check_output(["sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip()
    else:
        cpu = platform.processor() or platform.machine()
    return f"{cpu}, {os.cpu_count()} cores, {platform.system()}"


def play(elo: int, our_color: chess.Color, move_time: float, engine_path: Path, out: Path = GAMES) -> Path:
    sf = chess.engine.SimpleEngine.popen_uci(str(STOCKFISH))
    ours = chess.engine.SimpleEngine.popen_uci(str(engine_path))
    try:
        sf.configure({"UCI_LimitStrength": True, "UCI_Elo": elo})
        sf_version = sf.id.get("name", "Stockfish")
        our_name = ours.id.get("name", engine_path.name)

        board = chess.Board()
        limit = chess.engine.Limit(time=move_time)
        while not board.is_game_over(claim_draw=True):
            player = ours if board.turn == our_color else sf
            board.push(player.play(board, limit).move)
            print(board.peek(), end=" ", flush=True)
        print()
    finally:
        sf.quit()
        ours.quit()

    result = board.result(claim_draw=True)
    outcome = {"1-0": "white", "0-1": "black"}.get(result)
    ours_won = outcome == ("white" if our_color == chess.WHITE else "black")
    tag = "win" if ours_won else ("draw" if outcome is None else "loss")

    now = dt.datetime.now()
    stem = f"{now:%Y-%m-%dT%H%M%S}_sf{elo}_{tag}"
    sf_label = f"{sf_version} (UCI_Elo {elo})"

    game = chess.pgn.Game.from_board(board)
    game.headers.update({
        "Event": "Stockfish Destroyer match",
        "Date": f"{now:%Y.%m.%d}",
        "White": our_name if our_color == chess.WHITE else sf_label,
        "Black": sf_label if our_color == chess.WHITE else our_name,
        "WhiteElo": "?" if our_color == chess.WHITE else str(elo),
        "BlackElo": str(elo) if our_color == chess.WHITE else "?",
        "Result": result,
        "TimeControl": f"{move_time}s/move",
        "StockfishElo": str(elo),
        "Termination": board.outcome(claim_draw=True).termination.name.lower(),
    })

    meta = {
        "file": f"{stem}.pgn",
        "date": now.isoformat(timespec="seconds"),
        "stockfish_elo": elo,
        "stockfish_version": sf_version,
        "our_engine": our_name,
        "our_color": "white" if our_color == chess.WHITE else "black",
        "result": result,
        "outcome": tag,
        "plies": len(board.move_stack),
        "move_time_s": move_time,
        "engine_commit": git_commit(),
        "hardware": hardware(),
    }

    out.mkdir(parents=True, exist_ok=True)
    pgn_path = out / f"{stem}.pgn"
    pgn_path.write_text(str(game) + "\n")
    (out / f"{stem}.json").write_text(json.dumps(meta, indent=2) + "\n")
    rebuild_index(out)
    print(f"{tag.upper()} vs Stockfish {elo}: {result}  -> {pgn_path}")
    return pgn_path


def rebuild_index(out: Path = GAMES) -> None:
    """index.json is derived from the per-game .json files; on a merge conflict just rerun this."""
    metas = [json.loads(p.read_text()) for p in sorted(out.glob("*.json")) if p.name != "index.json"]
    (out / "index.json").write_text(json.dumps(metas, indent=2) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--elo", type=int, required=True, help="Stockfish UCI_Elo (1320-3190)")
    ap.add_argument("--color", choices=["white", "black"], default="white", help="our engine's color")
    ap.add_argument("--games", type=int, default=1)
    ap.add_argument("--time", type=float, default=MOVE_TIME, help="seconds per move (max 5)")
    ap.add_argument("--engine", type=Path, default=OUR_ENGINE, help="path to our UCI engine")
    ap.add_argument("--out", type=Path, default=GAMES, help="output dir (default games/; use a tmp dir for smoke tests)")
    args = ap.parse_args()
    if args.time > MOVE_TIME:
        ap.error("rules cap thinking time at 5 seconds per move")

    color = chess.WHITE if args.color == "white" else chess.BLACK
    for _ in range(args.games):
        play(args.elo, color, args.time, args.engine, args.out)


if __name__ == "__main__":
    main()
