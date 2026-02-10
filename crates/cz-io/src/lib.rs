//! # cz-io — The "Engine" of LACRIMOSA
//!
//! Single-threaded event loop that treats the disk as RAM.
//! Memory-mapped journal, ring buffer topology, raw io_uring I/O.

pub mod cursor;
pub mod event_loop;
pub mod ipc;
pub mod journal;
