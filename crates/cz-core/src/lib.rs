//! # cz-core — The "Physics" of LACRIMOSA
//!
//! Defines the physical data layout of reality. We do not use "Objects".
//! We use "Structs that are Bytes".
//!
//! The [`CausalEvent`] is the fundamental atom of the system — a packed,
//! zero-copy serializable struct with a deterministic ordering key.

#![no_std]

use core::cmp::Ordering;

/// The fundamental event atom of the LACRIMOSA sequencer.
///
/// This struct is `#[repr(C)]` — deterministic field layout, zero-copy safe.
/// Combined with rkyv's `unaligned` feature, it serializes directly to/from
/// wire format with no transformation.
///
/// # Memory Layout (32 bytes, C ABI)
///
/// | Offset | Size | Field            |
/// |--------|------|------------------|
/// | 0      | 8    | `lamport_ts`     |
/// | 8      | 4    | `node_id`        |
/// | 12     | 2    | `stream_id`      |
/// | 14     | 2    | `_pad`           |
/// | 16     | 8    | `payload_offset` |
/// | 24     | 4    | `checksum`       |
/// | 28     | 4    | (trailing pad)   |
///
/// # Ordering Key (The "Immutable Truth")
///
/// Events are ordered by `(lamport_ts, node_id, stream_id)`.
/// This 3-tuple defines the total causal order of the universe.
/// It is manually implemented via [`Ord`] and cannot be overridden.
///
/// # Zero-Copy
///
/// With `rkyv`, this struct is serialized and deserialized without any
/// copying or transformation. The archived representation IS the struct.
#[derive(Debug, Clone, Copy, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Debug))]
#[repr(C)]
pub struct CausalEvent {
    /// Lamport timestamp — the logical clock of causality.
    pub lamport_ts: u64,

    /// Node identifier — the observer that witnessed this event.
    pub node_id: u32,

    /// Stream identifier — the channel this event belongs to.
    pub stream_id: u16,

    /// Event flags (e.g. checkpoint bit).
    pub flags: u16,

    /// Byte offset of the payload blob, relative to the ring buffer start.
    pub payload_offset: u64,

    /// CRC32C checksum over the payload for integrity verification.
    pub checksum: u32,
}

pub const FLAG_CHECKPOINT: u16 = 0x1;

// =============================================================================
// The Immutable Truth: Manual Ord on (lamport_ts, node_id, stream_id)
// =============================================================================
//
// We implement Ord manually because the ordering key is a STRICT SUBSET
// of the struct fields. payload_offset, checksum, and _pad are NOT part
// of the causal order.

impl Ord for CausalEvent {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (self.lamport_ts, self.node_id, self.stream_id).cmp(&(
            other.lamport_ts,
            other.node_id,
            other.stream_id,
        ))
    }
}

impl PartialOrd for CausalEvent {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CausalEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CausalEvent {}

// =============================================================================
// Construction helpers
// =============================================================================

impl CausalEvent {
    /// Create a new `CausalEvent` with all fields specified.
    #[inline]
    pub const fn new(
        lamport_ts: u64,
        node_id: u32,
        stream_id: u16,
        payload_offset: u64,
        checksum: u32,
    ) -> Self {
        Self {
            lamport_ts,
            node_id,
            stream_id,
            flags: 0,
            payload_offset,
            checksum,
        }
    }

    /// Create a new `CausalEvent` with explicit flags.
    #[inline]
    pub const fn with_flags(
        lamport_ts: u64,
        node_id: u32,
        stream_id: u16,
        payload_offset: u64,
        checksum: u32,
        flags: u16,
    ) -> Self {
        Self {
            lamport_ts,
            node_id,
            stream_id,
            flags,
            payload_offset,
            checksum,
        }
    }

    /// Check if the checkpoint flag is set.
    #[inline]
    pub fn is_checkpoint(&self) -> bool {
        (self.flags & FLAG_CHECKPOINT) != 0
    }

    /// Returns the size of this struct in bytes.
    /// 32 bytes with `#[repr(C)]` deterministic layout.
    #[inline]
    pub const fn size_bytes() -> usize {
        core::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_size_is_32_bytes() {
        // 8 (u64) + 4 (u32) + 2 (u16) + 2 (pad) + 8 (u64) + 4 (u32) + 4 (trailing) = 32
        assert_eq!(CausalEvent::size_bytes(), 32);
    }

    #[test]
    fn test_ordering_by_lamport_ts_first() {
        let a = CausalEvent::new(1, 0, 0, 0, 0);
        let b = CausalEvent::new(2, 0, 0, 0, 0);
        assert!(a < b);
    }

    #[test]
    fn test_ordering_by_node_id_second() {
        let a = CausalEvent::new(1, 1, 0, 0, 0);
        let b = CausalEvent::new(1, 2, 0, 0, 0);
        assert!(a < b);
    }

    #[test]
    fn test_ordering_by_stream_id_third() {
        let a = CausalEvent::new(1, 1, 1, 0, 0);
        let b = CausalEvent::new(1, 1, 2, 0, 0);
        assert!(a < b);
    }

    #[test]
    fn test_payload_and_checksum_do_not_affect_ordering() {
        let a = CausalEvent::new(1, 1, 1, 999, 0xDEAD);
        let b = CausalEvent::new(1, 1, 1, 0, 0);
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn test_equality_ignores_payload_fields() {
        let a = CausalEvent::new(5, 3, 7, 100, 0xBEEF);
        let b = CausalEvent::new(5, 3, 7, 200, 0xCAFE);
        assert_eq!(a, b);
    }
}
