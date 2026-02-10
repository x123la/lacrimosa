//! # Journal — Memory-Mapped Persistent Storage
//!
//! The journal is a single contiguous file mapped into virtual memory via `mmap`.
//! It is split into two regions:
//!
//! - **Index Ring** (first 1 GiB): Fixed-size `CausalEvent` structs in a ring buffer.
//! - **Blob Storage** (remainder): Variable-length payload data.
//!
//! The file is pre-allocated at startup and never resized during operation.
//! All I/O goes through the kernel's page cache — we do not copy data.

use std::fs::{File, OpenOptions};
use std::path::Path;

use memmap2::MmapMut;

use cz_core::CausalEvent;

/// Default journal size: 100 GiB.
pub const DEFAULT_JOURNAL_SIZE: u64 = 100 * 1024 * 1024 * 1024;

/// Index ring size: 1 GiB.
/// Contains `INDEX_RING_CAPACITY` events.
pub const INDEX_RING_SIZE: usize = 1024 * 1024 * 1024;

/// Number of events that fit in the index ring.
pub const INDEX_RING_CAPACITY: usize = INDEX_RING_SIZE / CausalEvent::size_bytes();

/// The memory-mapped journal file.
///
/// Layout:
/// ```text
/// [0 .. INDEX_RING_SIZE)                → Index Ring (CausalEvent structs)
/// [INDEX_RING_SIZE .. journal_size)      → Blob Storage (payload bytes)
/// ```
pub struct Journal {
    /// The mutable memory map over the journal file.
    mmap: MmapMut,

    /// Total size of the journal in bytes.
    size: u64,

    /// The backing file (kept open for the lifetime of the journal).
    _file: File,
}

impl Journal {
    /// Open (or create) a journal file at `path` with the given `size`.
    ///
    /// The file is pre-allocated to `size` bytes and memory-mapped.
    /// If the file already exists, it is opened and mapped as-is.
    pub fn open(path: &Path, size: u64) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        // Pre-allocate the file to the requested size.
        file.set_len(size)?;

        // SAFETY: We own the file exclusively. No other process should
        // map the same file concurrently. The mmap is valid for the
        // lifetime of `_file`.
        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            mmap,
            size,
            _file: file,
        })
    }

    /// Returns a mutable slice over the Index Ring region.
    /// This region contains `CausalEvent` structs packed contiguously.
    #[inline]
    pub fn index_ring_mut(&mut self) -> &mut [u8] {
        &mut self.mmap[..INDEX_RING_SIZE]
    }

    /// Returns a slice over the Index Ring region.
    #[inline]
    pub fn index_ring(&self) -> &[u8] {
        &self.mmap[..INDEX_RING_SIZE]
    }

    /// Returns a mutable slice over the Blob Storage region.
    /// Payload data is written here, pointed to by `CausalEvent::payload_offset`.
    #[inline]
    pub fn blob_storage_mut(&mut self) -> &mut [u8] {
        &mut self.mmap[INDEX_RING_SIZE..]
    }

    /// Returns a slice over the Blob Storage region.
    #[inline]
    pub fn blob_storage(&self) -> &[u8] {
        &self.mmap[INDEX_RING_SIZE..]
    }

    /// Returns the total journal size in bytes.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the blob storage capacity in bytes.
    #[inline]
    pub fn blob_capacity(&self) -> usize {
        self.size as usize - INDEX_RING_SIZE
    }

    /// Write a `CausalEvent` at a specific slot index in the Index Ring.
    ///
    /// # Safety
    /// Caller must ensure `slot < INDEX_RING_CAPACITY`.
    #[inline]
    pub unsafe fn write_event_at(&mut self, slot: usize, event: &CausalEvent) {
        let offset = slot * CausalEvent::size_bytes();
        let dst = &mut self.mmap[offset..offset + CausalEvent::size_bytes()];
        // Zero-copy: reinterpret the struct as bytes and copy into mmap.
        let src = std::slice::from_raw_parts(
            event as *const CausalEvent as *const u8,
            CausalEvent::size_bytes(),
        );
        dst.copy_from_slice(src);
    }

    /// Read a `CausalEvent` from a specific slot index in the Index Ring.
    ///
    /// # Safety
    /// Caller must ensure `slot < INDEX_RING_CAPACITY` and that valid
    /// data was previously written at this slot.
    #[inline]
    pub unsafe fn read_event_at(&self, slot: usize) -> CausalEvent {
        let offset = slot * CausalEvent::size_bytes();
        let src = &self.mmap[offset..offset + CausalEvent::size_bytes()];
        std::ptr::read(src.as_ptr() as *const CausalEvent)
    }

    /// Flush the mmap to disk.
    pub fn flush(&self) -> std::io::Result<()> {
        self.mmap.flush()
    }
}
