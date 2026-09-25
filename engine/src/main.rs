//! Stockfish Destroyer: UCI engine.
//! v0 baseline: iterative-deepening alpha-beta with material-only eval.

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{CastlingMode, Chess, Color, Move, Position, Role};
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

const MATE: i32 = 100_000;
const INF: i32 = 1_000_000;

fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn => 100,
        Role::Knight => 320,
        Role::Bishop => 330,
        Role::Rook => 500,
        Role::Queen => 900,
        Role::King => 0,
    }
}

/// Static eval from the side to move's perspective.
fn evaluate(pos: &Chess) -> i32 {
    let board = pos.board();
    let mut score = 0;
    for (_sq, piece) in board.iter() {
        let v = piece_value(piece.role);
        score += if piece.color == Color::White { v } else { -v };
    }
    if pos.turn() == Color::White { score } else { -score }
}

struct Search {
    deadline: Instant,
    nodes: u64,
    stopped: bool,
}

impl Search {
    fn negamax(&mut self, pos: &Chess, depth: u32, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        self.nodes += 1;
        if self.nodes & 2047 == 0 && Instant::now() >= self.deadline {
            self.stopped = true;
        }
        if self.stopped {
            return 0;
        }
        let moves = pos.legal_moves();
        if moves.is_empty() {
            return if pos.is_check() { -MATE + ply } else { 0 };
        }
        if depth == 0 {
            return evaluate(pos);
        }
        let mut ordered: Vec<Move> = moves.into_iter().collect();
        ordered.sort_by_key(|m| -m.capture().map_or(0, piece_value));
        for m in ordered {
            let mut child = pos.clone();
            child.play_unchecked(&m);
            let score = -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1);
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }

    fn best_move(&mut self, pos: &Chess) -> Option<Move> {
        let mut moves: Vec<Move> = pos.legal_moves().into_iter().collect();
        let mut best = moves.first().cloned();
        for depth in 1..64 {
            let mut alpha = -INF;
            let mut depth_best = None;
            for m in &moves {
                let mut child = pos.clone();
                child.play_unchecked(m);
                let score = -self.negamax(&child, depth - 1, -INF, -alpha, 1);
                if self.stopped {
                    break;
                }
                if score > alpha {
                    alpha = score;
                    depth_best = Some(m.clone());
                }
            }
            if self.stopped {
                break;
            }
            if let Some(m) = depth_best {
                println!("info depth {depth} score cp {alpha} nodes {}", self.nodes);
                // search the previous best first on the next iteration
                moves.retain(|x| x != &m);
                moves.insert(0, m.clone());
                best = Some(m);
            }
        }
        best
    }
}

fn parse_position(args: &[&str]) -> Chess {
    let mut pos = Chess::default();
    let moves_at = args.iter().position(|&a| a == "moves");
    if args.first() == Some(&"fen") {
        let fen_str = args[1..moves_at.unwrap_or(args.len())].join(" ");
        if let Ok(fen) = fen_str.parse::<Fen>() {
            if let Ok(p) = fen.into_position(CastlingMode::Standard) {
                pos = p;
            }
        }
    }
    if let Some(i) = moves_at {
        for uci in &args[i + 1..] {
            let Ok(uci) = uci.parse::<UciMove>() else { break };
            let Ok(m) = uci.to_move(&pos) else { break };
            pos.play_unchecked(&m);
        }
    }
    pos
}

/// Pick a time budget from `go` arguments, always under the 5 s/move rule.
fn think_time(args: &[&str], turn: Color) -> Duration {
    let get = |key: &str| {
        args.iter()
            .position(|&a| a == key)
            .and_then(|i| args.get(i + 1))
            .and_then(|v| v.parse::<u64>().ok())
    };
    let ms = if let Some(mt) = get("movetime") {
        mt
    } else {
        let (t, inc) = if turn == Color::White { (get("wtime"), get("winc")) } else { (get("btime"), get("binc")) };
        t.map_or(5000, |t| t / 30 + inc.unwrap_or(0))
    };
    // safety margin for process/IPC overhead
    Duration::from_millis(ms.min(5000).saturating_sub(100).max(10))
}

fn main() {
    let stdin = io::stdin();
    let mut pos = Chess::default();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.first().copied() {
            Some("uci") => {
                println!("id name Destroyer {}", env!("CARGO_PKG_VERSION"));
                println!("id author stockfish-destroyer team");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("ucinewgame") => pos = Chess::default(),
            Some("position") => pos = parse_position(&parts[1..]),
            Some("go") => {
                let mut s = Search { deadline: Instant::now() + think_time(&parts[1..], pos.turn()), nodes: 0, stopped: false };
                match s.best_move(&pos) {
                    Some(m) => println!("bestmove {}", m.to_uci(CastlingMode::Standard)),
                    None => println!("bestmove 0000"),
                }
            }
            Some("quit") => break,
            _ => {}
        }
        io::stdout().flush().ok();
    }
}
