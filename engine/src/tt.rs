//! Transposition table: fixed-size, replace-if-deeper-or-same-key, with mate scores stored
//! relative to the node so they stay valid when the same position is reached at another ply.

use shakmaty::Move;

use crate::MATE;

const MATE_BOUND: i32 = MATE - 1000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub key: u64,
    pub score: i32,
    pub depth: u8,
    pub bound: Bound,
    pub best: Option<Move>,
}

pub struct Table {
    entries: Vec<Option<Entry>>,
    mask: usize,
}

impl Table {
    /// `log2_size` = 21 gives 2M entries (~64 MB).
    pub fn new(log2_size: u32) -> Self {
        let n = 1usize << log2_size;
        Table { entries: vec![None; n], mask: n - 1 }
    }

    pub fn clear(&mut self) {
        self.entries.iter_mut().for_each(|e| *e = None);
    }

    pub fn probe(&self, key: u64, ply: i32) -> Option<Entry> {
        let e = self.entries[(key as usize) & self.mask].as_ref()?;
        if e.key != key {
            return None;
        }
        Some(Entry { score: from_tt(e.score, ply), ..e.clone() })
    }

    pub fn store(&mut self, key: u64, depth: u32, score: i32, bound: Bound, best: Option<Move>, ply: i32) {
        let slot = &mut self.entries[(key as usize) & self.mask];
        let replace = match slot {
            None => true,
            Some(e) => e.key != key || depth as u8 >= e.depth || bound == Bound::Exact,
        };
        if replace {
            *slot = Some(Entry { key, score: to_tt(score, ply), depth: depth.min(255) as u8, bound, best });
        }
    }
}

/// Mate scores are "plies from root"; store them as "plies from this node".
fn to_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_BOUND {
        score + ply
    } else if score < -MATE_BOUND {
        score - ply
    } else {
        score
    }
}

fn from_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_BOUND {
        score - ply
    } else if score < -MATE_BOUND {
        score + ply
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_mate_adjustment() {
        let mut t = Table::new(10);
        t.store(42, 5, 123, Bound::Exact, None, 3);
        let e = t.probe(42, 7).unwrap();
        assert_eq!((e.score, e.depth, e.bound), (123, 5, Bound::Exact));
        assert!(t.probe(43, 0).is_none());
        // mate in 2 plies from a node at ply 4 (score MATE-6 from the root) probed at ply 10
        t.store(7, 3, MATE - 6, Bound::Exact, None, 4);
        assert_eq!(t.probe(7, 10).unwrap().score, MATE - 12);
        t.store(8, 3, -(MATE - 6), Bound::Exact, None, 4);
        assert_eq!(t.probe(8, 10).unwrap().score, -(MATE - 12));
    }

    #[test]
    fn deeper_entry_is_not_replaced_by_shallower_bound() {
        let mut t = Table::new(10);
        t.store(1, 8, 50, Bound::Lower, None, 0);
        t.store(1, 2, 10, Bound::Upper, None, 0);
        assert_eq!(t.probe(1, 0).unwrap().depth, 8);
        t.store(1, 2, 10, Bound::Exact, None, 0); // exact always wins
        assert_eq!(t.probe(1, 0).unwrap().depth, 2);
    }
}
