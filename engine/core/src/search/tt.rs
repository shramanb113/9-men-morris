use crate::types::Move;

/// Which side of the true score a stored value represents, from an
/// alpha-beta search that didn't finish exploring every branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// The stored score is the exact value of the node.
    Exact,
    /// The stored score is a lower bound (search failed high / beta cutoff).
    Lower,
    /// The stored score is an upper bound (search failed low).
    Upper,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    /// Full 64-bit zobrist key, stored alongside the entry so a same-slot,
    /// different-key collision is detected as a miss instead of returned as
    /// a stale hit.
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub bound: Bound,
    pub best_move: Option<Move>,
}

/// Fixed-size transposition table indexed by `key & (capacity - 1)`, so
/// `capacity` is rounded up to a power of two. Bounded, predictable memory
/// footprint instead of an unbounded `HashMap` — matters for a WASM target.
pub struct TranspositionTable {
    slots: Vec<Option<TTEntry>>,
    mask: u64,
}

impl TranspositionTable {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(1);
        Self { slots: vec![None; capacity], mask: (capacity - 1) as u64 }
    }

    fn index(&self, key: u64) -> usize {
        (key & self.mask) as usize
    }

    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        match &self.slots[self.index(key)] {
            Some(entry) if entry.key == key => Some(entry),
            _ => None,
        }
    }

    /// Depth-preferred replacement: only overwrites an existing entry if the
    /// new one was searched at least as deep, so a shallow early
    /// iterative-deepening pass can't evict a deeper result.
    pub fn store(&mut self, key: u64, entry: TTEntry) {
        let idx = self.index(key);
        let replace = match &self.slots[idx] {
            Some(existing) => entry.depth >= existing.depth,
            None => true,
        };
        if replace {
            self.slots[idx] = Some(entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: u64, depth: u8, score: i32) -> TTEntry {
        TTEntry { key, depth, score, bound: Bound::Exact, best_move: None }
    }

    #[test]
    fn store_then_probe_round_trips() {
        let mut tt = TranspositionTable::new(16);
        tt.store(5, entry(5, 3, 42));
        assert_eq!(tt.probe(5).unwrap().score, 42);
    }

    #[test]
    fn probing_an_absent_key_misses() {
        let tt = TranspositionTable::new(16);
        assert!(tt.probe(5).is_none());
    }

    #[test]
    fn a_different_key_at_the_same_slot_is_a_miss_not_a_stale_hit() {
        let mut tt = TranspositionTable::new(16); // mask = 15
        tt.store(5, entry(5, 3, 42));
        // 5 and 21 both map to slot 5 (5 & 15 == 21 & 15) but differ as keys.
        assert!(tt.probe(21).is_none());
    }

    #[test]
    fn shallower_search_does_not_evict_a_deeper_entry() {
        let mut tt = TranspositionTable::new(16);
        tt.store(5, entry(5, 5, 100));
        tt.store(5, entry(5, 2, 1));
        assert_eq!(tt.probe(5).unwrap().score, 100);
    }

    #[test]
    fn equal_or_deeper_search_does_replace() {
        let mut tt = TranspositionTable::new(16);
        tt.store(5, entry(5, 2, 1));
        tt.store(5, entry(5, 5, 100));
        assert_eq!(tt.probe(5).unwrap().score, 100);
    }
}
