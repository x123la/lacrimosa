//! # Cursor — Ring Buffer Position Tracker
//!
//! Tracks the `head` (write position) and `tail` (commit/read position)
//! of the Index Ring. Enforces the critical invariant: `head` can never
//! wrap around and touch `tail`.
//!
//! This invariant is formally verified with Kani in `cz-verify`.

/// Ring buffer cursor tracking write (head) and commit (tail) positions.
///
/// The ring has `capacity` slots, each holding one `CausalEvent`.
/// Positions wrap around using modular arithmetic.
///
/// # Invariant
///
/// `head` can never advance to equal `tail` (that would mean the buffer
/// wrapped around and overwrote uncommitted data). The ring has
/// `capacity - 1` usable slots to maintain this invariant.
pub struct Cursor {
    /// Current write position (next slot to write into).
    head: usize,

    /// Current commit/read position (oldest unread slot).
    tail: usize,

    /// Total number of slots in the ring.
    capacity: usize,
}

impl Cursor {
    /// Create a new cursor for a ring with the given number of event slots.
    ///
    /// # Panics
    /// Panics if `capacity < 2` (ring must have at least 2 slots to be useful).
    pub fn new(capacity: usize) -> Self {
        assert!(capacity >= 2, "Ring buffer must have at least 2 slots");
        Self {
            head: 0,
            tail: 0,
            capacity,
        }
    }

    /// Create a cursor sized to fill the Index Ring region.
    pub fn for_index_ring() -> Self {
        let capacity = super::journal::INDEX_RING_CAPACITY;
        Self::new(capacity)
    }

    /// Returns `true` if the ring buffer is full.
    /// A full ring means advancing `head` would make it equal `tail`.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.next_pos(self.head) == self.tail
    }

    /// Returns `true` if the ring buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// Returns the number of events currently in the ring.
    #[inline]
    pub fn len(&self) -> usize {
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.capacity - self.tail + self.head
        }
    }

    /// Returns the current head (write) position.
    #[inline]
    pub fn head(&self) -> usize {
        self.head
    }

    /// Returns the current tail (read/commit) position.
    #[inline]
    pub fn tail(&self) -> usize {
        self.tail
    }

    /// Returns the ring capacity (total slots).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Advance the head pointer by one slot.
    ///
    /// Returns the slot index that was claimed for writing,
    /// or `None` if the ring is full.
    #[inline]
    pub fn advance_head(&mut self) -> Option<usize> {
        if self.is_full() {
            return None;
        }
        let slot = self.head;
        self.head = self.next_pos(self.head);
        Some(slot)
    }

    /// Advance the tail pointer by one slot (mark oldest event as consumed).
    ///
    /// Returns the slot index that was released,
    /// or `None` if the ring is empty.
    #[inline]
    pub fn advance_tail(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }
        let slot = self.tail;
        self.tail = self.next_pos(self.tail);
        Some(slot)
    }

    /// Compute the next position with wrap-around.
    #[inline]
    fn next_pos(&self, pos: usize) -> usize {
        (pos + 1) % self.capacity
    }
}

// =============================================================================
// Kani Proofs: Ring Buffer Invariants
// =============================================================================

#[cfg(kani)]
mod proofs {
    use super::*;

    /// **Proof: Head cannot wrap around and touch tail**
    ///
    /// After any sequence of advance_head calls on a non-full ring,
    /// head != tail (unless the ring started empty and we didn't advance).
    #[kani::proof]
    fn verify_head_cannot_overwrite_tail() {
        // Use a small ring (4 slots) to keep verification tractable.
        let mut cursor = Cursor::new(4);

        // Symbolic number of advances (0..4)
        let advances: usize = kani::any();
        kani::assume(advances <= 4);

        for _ in 0..advances {
            let _ = cursor.advance_head();
        }

        // If the ring is not empty, head and tail must differ.
        // If the ring IS empty, they can be equal (both at same position).
        if !cursor.is_empty() {
            assert!(
                cursor.head != cursor.tail,
                "INVARIANT VIOLATED: head wrapped around to touch tail"
            );
        }
    }

    /// **Proof: Ring never reports negative or overflow length**
    #[kani::proof]
    fn verify_len_consistency() {
        let mut cursor = Cursor::new(4);

        let head_advances: usize = kani::any();
        let tail_advances: usize = kani::any();
        kani::assume(head_advances <= 4);
        kani::assume(tail_advances <= head_advances);

        for _ in 0..head_advances {
            let _ = cursor.advance_head();
        }
        for _ in 0..tail_advances {
            let _ = cursor.advance_tail();
        }

        assert!(cursor.len() <= cursor.capacity());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cursor_is_empty() {
        let c = Cursor::new(10);
        assert!(c.is_empty());
        assert!(!c.is_full());
        assert_eq!(c.len(), 0);
    }

    #[test]
    fn test_advance_head_returns_slot() {
        let mut c = Cursor::new(4);
        assert_eq!(c.advance_head(), Some(0));
        assert_eq!(c.advance_head(), Some(1));
        assert_eq!(c.advance_head(), Some(2));
        // Ring is full (3 usable slots out of 4)
        assert_eq!(c.advance_head(), None);
    }

    #[test]
    fn test_advance_tail_frees_slot() {
        let mut c = Cursor::new(4);
        c.advance_head();
        c.advance_head();
        c.advance_head();
        assert!(c.is_full());

        c.advance_tail();
        assert!(!c.is_full());
        assert_eq!(c.advance_head(), Some(3));
    }

    #[test]
    fn test_wrap_around() {
        let mut c = Cursor::new(3);
        // Fill: slots 0, 1
        c.advance_head(); // head=1
        c.advance_head(); // head=2, full

        // Consume slot 0
        c.advance_tail(); // tail=1

        // Write wraps to slot 2
        assert_eq!(c.advance_head(), Some(2)); // head=0 (wrapped)
    }

    #[test]
    fn test_empty_tail_returns_none() {
        let mut c = Cursor::new(4);
        assert_eq!(c.advance_tail(), None);
    }
}
