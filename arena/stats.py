"""Per-level campaign statistics: W-D-L, win ratio and average game length (the addendum's tie-break).

Reads games/*.json (the same metadata the replay UI uses). At Stockfish's maximum Elo (3190) the
score is `wins / games` and the average number of moves; both are highlighted here and in the GUI.
"""

import argparse
import json
from collections import defaultdict
from pathlib import Path

from arena.play import GAMES

MAX_ELO = 3190


def level_stats(games_dir: Path = GAMES) -> list[dict]:
    per: dict[int, list[dict]] = defaultdict(list)
    for p in sorted(games_dir.glob("*.json")):
        if p.name == "index.json":
            continue
        m = json.loads(p.read_text())
        per[m["stockfish_elo"]].append(m)
    rows = []
    for elo, ms in sorted(per.items(), reverse=True):
        wins = [m for m in ms if m["outcome"] == "win"]
        draws = sum(m["outcome"] == "draw" for m in ms)
        losses = sum(m["outcome"] == "loss" for m in ms)
        avg = lambda xs: round(sum(x["plies"] for x in xs) / 2 / len(xs), 1) if xs else None
        rows.append({
            "elo": elo, "games": len(ms), "wins": len(wins), "draws": draws, "losses": losses,
            "win_ratio": round(len(wins) / len(ms), 3),
            "avg_moves": avg(ms), "avg_moves_wins": avg(wins),
            "max_level": elo == MAX_ELO,
        })
    return rows


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--games", type=Path, default=GAMES)
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    args = ap.parse_args()
    rows = level_stats(args.games)
    if args.json:
        print(json.dumps(rows, indent=2))
        return
    print(f"{'Elo':>5} {'W-D-L':>9} {'ratio':>6} {'avg moves':>10} {'avg (wins)':>11}")
    for r in rows:
        tag = "  <- MAX level: win ratio + average moves decide" if r["max_level"] else ""
        print(f"{r['elo']:>5} {r['wins']:>3}-{r['draws']}-{r['losses']:<3} {r['win_ratio']:>6.0%} "
              f"{r['avg_moves'] if r['avg_moves'] is not None else '-':>10} "
              f"{r['avg_moves_wins'] if r['avg_moves_wins'] is not None else '-':>11}{tag}")
    top = next((r for r in rows if r["max_level"]), None)
    if top:
        print(f"\nAt 3190: {top['wins']}/{top['games']} wins = {top['win_ratio']:.0%}, {top['avg_moves']} moves on average")
    else:
        print("\n3190 not reached yet; highest level played:", rows[0]["elo"] if rows else "-")


if __name__ == "__main__":
    main()
