"""Always-on campaign: keeps playing official games (we are white) vs Stockfish in parallel slots.

Reads campaign.json before every game, so the loop can change levels or flags without a restart:
  slots          list of Stockfish Elos, one game slot per entry
  engine         frozen copy of the latest kept engine (promote with scripts/promote.sh)
  move_time      seconds per move (max 5)
  abort_hopeless resign clearly lost games early to free the slot (saved as losses)
  out            output dir (default games/; only change for smoke tests)
Results are appended to campaign.log. Stop with Ctrl-C.
"""

import datetime as dt
import json
import threading
import time
from pathlib import Path

import chess

from arena.play import ROOT, play

CONFIG = ROOT / "campaign.json"
LOG = ROOT / "campaign.log"
lock = threading.Lock()


def log(msg: str) -> None:
    line = f"{dt.datetime.now():%H:%M:%S} {msg}"
    with lock:
        print(line, flush=True)
        with LOG.open("a") as f:
            f.write(line + "\n")


def config() -> dict:
    return json.loads(CONFIG.read_text())


def slot(i: int) -> None:
    while True:
        cfg = config()
        if i >= len(cfg["slots"]):
            time.sleep(30)
            continue
        elo = cfg["slots"][i]
        engine = ROOT / cfg["engine"]
        if not engine.exists():
            log(f"slot {i}: {engine} missing, waiting")
            time.sleep(30)
            continue
        try:
            pgn = play(elo, chess.WHITE, min(cfg.get("move_time", 5.0), 5.0), engine,
                       out=ROOT / cfg.get("out", "games"), abort_hopeless=cfg.get("abort_hopeless", False))
            log(f"slot {i}: {pgn.stem}")
        except Exception as e:  # keep the slot alive
            log(f"slot {i}: error {e!r}")
            time.sleep(10)


def main() -> None:
    cfg = config()
    log(f"campaign start: slots={cfg['slots']} engine={cfg['engine']}")
    threads = [threading.Thread(target=slot, args=(i,), daemon=True) for i in range(8)]
    for t in threads:
        t.start()
        time.sleep(2)  # stagger starts
    try:
        while True:
            time.sleep(60)
    except KeyboardInterrupt:
        log("campaign stopped")


if __name__ == "__main__":
    main()
