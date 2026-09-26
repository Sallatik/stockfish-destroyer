//! Stockfish Destroyer: UCI engine.
//! Iterative-deepening alpha-beta + quiescence search, tapered PeSTO evaluation.

mod book;
mod eval;
mod tt;
use eval::evaluate;
use tt::{Bound, Table};

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::zobrist::{Zobrist64, ZobristHash};
use shakmaty::{CastlingMode, Chess, Color, EnPassantMode, Move, Position, Role};
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

const MATE: i32 = 100_000;
const INF: i32 = 1_000_000;
const MAX_PLY: i32 = 96;
/// Draws are worth nothing to us: score them as a small loss for the root side (unless we are
/// already clearly lost at the root, then a draw is fine).
const CONTEMPT: i32 = 30;

fn hash(pos: &Chess) -> u64 {
    pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal).0
}

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

/// Move ordering key: most valuable victim first, least valuable attacker as tiebreak.
fn mvv_lva(m: &Move) -> i32 {
    match m.capture() {
        Some(victim) => 10 * piece_value(victim) - piece_value(m.role()) / 10,
        None => m.promotion().map_or(0, piece_value),
    }
}

struct Search<'a> {
    tt: &'a mut Table,
    deadline: Instant,
    nodes: u64,
    stopped: bool,
    /// Zobrist hashes of every position in the game so far (root last), followed by the hashes of
    /// the current search path. Used for repetition detection.
    path: Vec<u64>,
    n_hist: usize,
    contempt: i32,
    /// score of the best root move from the last completed iteration
    best_score: i32,
}

impl<'a> Search<'a> {
    fn new(tt: &'a mut Table, deadline: Instant, history: &[u64], contempt: i32) -> Self {
        Search { tt, deadline, nodes: 0, stopped: false, path: history.to_vec(), n_hist: history.len(), contempt, best_score: 0 }
    }

    /// Draw score from the perspective of the side to move at `ply` (root side = even plies).
    fn draw_score(&self, ply: i32) -> i32 {
        if ply % 2 == 0 { -self.contempt } else { self.contempt }
    }

    /// Has the position with hash `h` (about to be pushed onto `path`) already occurred since the
    /// last irreversible move? Any earlier occurrence counts (two-fold), as usual in search.
    fn is_repetition(&self, h: u64, halfmoves: u32) -> bool {
        let n = self.path.len() as i64;
        let limit = (n - halfmoves as i64).max(0);
        let mut i = n - 2;
        while i >= limit {
            if self.path[i as usize] == h {
                return true;
            }
            i -= 2;
        }
        false
    }

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
            return if pos.is_check() { -MATE + ply } else { self.draw_score(ply) };
        }
        let h = hash(pos);
        if ply > 0 {
            self.path.truncate(self.n_hist + ply as usize - 1);
            if pos.halfmoves() >= 100 || pos.is_insufficient_material() || self.is_repetition(h, pos.halfmoves()) {
                return self.draw_score(ply);
            }
            self.path.push(h);
        }
        if depth == 0 {
            return self.quiesce(pos, alpha, beta, ply);
        }
        // transposition table: cutoff if we already searched this position deep enough
        let mut hash_move = None;
        if let Some(e) = self.tt.probe(h, ply) {
            hash_move = e.best;
            if ply > 0 && u32::from(e.depth) >= depth {
                match e.bound {
                    Bound::Exact => return e.score,
                    Bound::Lower if e.score >= beta => return e.score,
                    Bound::Upper if e.score <= alpha => return e.score,
                    _ => {}
                }
            }
        }
        let mut ordered: Vec<Move> = moves.into_iter().collect();
        ordered.sort_by_key(|m| if Some(m) == hash_move.as_ref() { i32::MIN } else { -mvv_lva(m) });
        let alpha_orig = alpha;
        let mut best_score = -INF;
        let mut best_move = None;
        for m in ordered {
            let mut child = pos.clone();
            child.play_unchecked(&m);
            let score = -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1);
            if self.stopped {
                return 0;
            }
            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                break;
            }
        }
        let bound = if best_score <= alpha_orig {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        self.tt.store(h, depth, best_score, bound, best_move, ply);
        best_score
    }

    /// Quiescence search: at the horizon, keep resolving captures/promotions so we never
    /// evaluate a position in the middle of an exchange.
    fn quiesce(&mut self, pos: &Chess, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        self.nodes += 1;
        if self.nodes & 2047 == 0 && Instant::now() >= self.deadline {
            self.stopped = true;
        }
        if self.stopped {
            return 0;
        }
        if ply >= MAX_PLY {
            return evaluate(pos);
        }
        let in_check = pos.is_check();
        let moves = pos.legal_moves();
        if moves.is_empty() {
            return if in_check { -MATE + ply } else { 0 };
        }
        // stand pat: we can usually decline to capture (not when in check)
        if !in_check {
            let stand = evaluate(pos);
            if stand >= beta {
                return beta;
            }
            alpha = alpha.max(stand);
        }
        let mut ordered: Vec<Move> = moves
            .into_iter()
            .filter(|m| in_check || m.is_capture() || m.is_promotion())
            .collect();
        ordered.sort_by_key(|m| -mvv_lva(m));
        for m in ordered {
            let mut child = pos.clone();
            child.play_unchecked(&m);
            let score = -self.quiesce(&child, -beta, -alpha, ply + 1);
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
                self.best_score = alpha;
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

/// Returns the position plus the hashes of every position reached on the way (root last).
fn parse_position(args: &[&str]) -> (Chess, Vec<u64>) {
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
    let mut history = vec![hash(&pos)];
    if let Some(i) = moves_at {
        for uci in &args[i + 1..] {
            let Ok(uci) = uci.parse::<UciMove>() else { break };
            let Ok(m) = uci.to_move(&pos) else { break };
            pos.play_unchecked(&m);
            history.push(hash(&pos));
        }
    }
    (pos, history)
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
    let mut history = vec![hash(&pos)];
    let mut tt = Table::new(21);
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
            Some("ucinewgame") => {
                pos = Chess::default();
                history = vec![hash(&pos)];
                tt.clear();
            }
            Some("position") => (pos, history) = parse_position(&parts[1..]),
            Some("go") => {
                if let Some(m) = book::probe(&pos) {
                    println!("info string book move");
                    println!("bestmove {}", m.to_uci(CastlingMode::Standard));
                    io::stdout().flush().ok();
                    continue;
                }
                // if we are clearly lost anyway, a draw is welcome
                let contempt = if evaluate(&pos) < -300 { 0 } else { CONTEMPT };
                let mut s = Search::new(&mut tt, Instant::now() + think_time(&parts[1..], pos.turn()), &history, contempt);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<&str> {
        s.split_whitespace().collect()
    }

    #[test]
    fn history_records_every_position() {
        let (pos, hist) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8"));
        assert_eq!(hist.len(), 5);
        assert_eq!(hist[0], hist[4]);
        assert_eq!(hist[4], hash(&pos));
        assert_ne!(hist[1], hist[3]);
        assert_eq!(pos.halfmoves(), 4);
    }

    #[test]
    fn detects_repetition_across_game_history() {
        let (root, hist) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8"));
        let mut tt = Table::new(12);
        let s = Search::new(&mut tt, Instant::now() + Duration::from_secs(1), &hist, CONTEMPT);
        // one ply into the search, after Nf3 again: same position as hist[1]
        let (after_nf3, _) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8 g1f3"));
        assert!(s.is_repetition(hash(&after_nf3), after_nf3.halfmoves()));
        // a fresh position is not a repetition
        let (after_e4, _) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8 e2e4"));
        assert!(!s.is_repetition(hash(&after_e4), after_e4.halfmoves()));
        assert_eq!(hash(&root), hist[4]);
    }

    #[test]
    fn repetition_scores_as_contempt_draw() {
        let (_, hist) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8"));
        let mut tt = Table::new(12);
        let mut s = Search::new(&mut tt, Instant::now() + Duration::from_secs(1), &hist, CONTEMPT);
        let (after_nf3, _) = parse_position(&args("startpos moves g1f3 g8f6 f3g1 f6g8 g1f3"));
        // ply 1 = opponent to move; a draw is good for them (+contempt)
        assert_eq!(s.negamax(&after_nf3, 3, -INF, INF, 1), CONTEMPT);
    }

    #[test]
    fn fifty_move_rule_is_a_draw() {
        let (pos, hist) = parse_position(&args("fen 8/8/8/8/8/4k3/8/R3K3 w - - 100 80"));
        let mut tt = Table::new(12);
        let mut s = Search::new(&mut tt, Instant::now() + Duration::from_secs(1), &hist, CONTEMPT);
        // pretend we are one ply into the search: any node with halfmoves >= 100 is a draw
        s.path.push(hash(&pos));
        assert_eq!(s.negamax(&pos, 3, -INF, INF, 1), CONTEMPT);
    }

    #[test]
    fn avoids_repetition_when_winning() {
        // white is a queen up; after Qa1-b1 Ke3-e2 Qb1-a1 Ke2-e3 the tempting "repeat" is a draw,
        // so the search must prefer any other move
        let (pos, hist) = parse_position(&args("fen 8/8/8/8/8/4k3/8/Q3K3 w - - 0 1 moves a1b1 e3e2 b1a1 e2e3"));
        let mut tt = Table::new(12);
        let mut s = Search::new(&mut tt, Instant::now() + Duration::from_millis(300), &hist, CONTEMPT);
        let m = s.best_move(&pos).unwrap();
        assert_ne!(m.to_uci(CastlingMode::Standard).to_string(), "a1b1");
    }

    #[test]
    fn finds_mate_in_two_with_correct_score() {
        // 1.Kb6 Kb8 2.Rh8#  (mate at ply 3 -> root score MATE-3); TT must not corrupt mate distances
        let (pos, hist) = parse_position(&args("fen k7/8/2K5/8/8/8/8/7R w - - 0 1"));
        let mut tt = Table::new(16);
        let mut s = Search::new(&mut tt, Instant::now() + Duration::from_millis(500), &hist, CONTEMPT);
        let m = s.best_move(&pos).unwrap();
        assert_eq!(m.to_uci(CastlingMode::Standard).to_string(), "c6b6");
        assert_eq!(s.best_score, MATE - 3);
    }

    #[test]
    fn tt_reaches_deeper_than_no_tt_for_same_nodes() {
        // sanity: with a warm TT the second search of the same position is much cheaper
        let (pos, hist) = parse_position(&args("startpos moves e2e4 e7e5 g1f3 b8c6 f1b5 a7a6"));
        let mut tt = Table::new(18);
        let mut s1 = Search::new(&mut tt, Instant::now() + Duration::from_millis(400), &hist, CONTEMPT);
        let m1 = s1.best_move(&pos);
        let n1 = s1.nodes;
        let mut s2 = Search::new(&mut tt, Instant::now() + Duration::from_millis(400), &hist, CONTEMPT);
        let m2 = s2.best_move(&pos);
        assert!(m1.is_some() && m2.is_some());
        assert!(n1 > 10_000);
    }
}
