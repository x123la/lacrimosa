//! # Event Loop — Refactored io_uring UDP Receiver (Zero-Copy & Pipelined)
//!
//! High-performance single-threaded event loop.
//! Uses io_uring to receive UDP packets directly into mmap'd blob storage.
//! Implements hardware-accelerated checksum verification and network input validation.

use std::net::UdpSocket;
use std::os::fd::AsRawFd;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use crc32fast::Hasher;
use io_uring::{opcode, types, IoUring};

use cz_core::CausalEvent;

use crate::cursor::Cursor;
use crate::ipc::IpcServer;
use crate::journal::Journal;

/// Maximum UDP packet size we expect to receive.
const MAX_PACKET_SIZE: usize = 65535;

/// Number of concurrent receive operations to keep in flight.
const PIPELINE_DEPTH: usize = 16;

/// Global statistics for telemetry.
pub static EVENTS_PROCESSED: AtomicU64 = AtomicU64::new(0);
pub static BYTES_PROCESSED: AtomicU64 = AtomicU64::new(0);

/// Global monotonic Lamport timestamp counter.
static LAMPORT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Configuration for the event loop.
pub struct EventLoopConfig {
    pub bind_addr: String,
    pub ring_depth: u32,
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:9000".to_string(),
            ring_depth: 256,
        }
    }
}

pub struct EventLoop {
    ring: IoUring,
    socket: UdpSocket,
    /// Next available offset in blob storage for receiving.
    next_blob_offset: usize,
    /// IPC server for real-time notifications.
    ipc: Option<IpcServer>,
}

impl EventLoop {
    pub fn new(config: &EventLoopConfig) -> std::io::Result<Self> {
        let ring = IoUring::new(config.ring_depth)?;
        let socket = UdpSocket::bind(&config.bind_addr)?;
        socket.set_nonblocking(true)?;

        let ipc = IpcServer::start("/tmp/cz-io.sock").ok();

        Ok(Self {
            ring,
            socket,
            next_blob_offset: 0,
            ipc,
        })
    }

    pub fn run(&mut self, journal: &mut Journal, cursor: &mut Cursor) -> std::io::Result<()> {
        let fd = types::Fd(self.socket.as_raw_fd());
        let _blob_capacity = journal.blob_capacity();

        // Track the blob storage offsets assigned to each in-flight request.
        // We use user_data in io_uring to index into this array.
        let mut in_flight_offsets = [0usize; PIPELINE_DEPTH];

        // === INITIAL SUBMISSION: Fill the pipeline ===
        for i in 0..PIPELINE_DEPTH {
            self.submit_recv(fd, journal, i, &mut in_flight_offsets)?;
        }

        loop {
            // Wait for at least 1 completion.
            self.ring.submit_and_wait(1)?;

            // 1. COLLECT COMPLETIONS: Decouple from &mut self to satisfy borrow checker.
            // We use a small local buffer to avoid heap allocation in the hot loop.
            let mut completed_slots = [None::<(usize, i32)>; PIPELINE_DEPTH];
            let mut count = 0;

            {
                let mut completions = self.ring.completion();
                while let Some(cqe) = completions.next() {
                    if count < PIPELINE_DEPTH {
                        completed_slots[count] = Some((cqe.user_data() as usize, cqe.result()));
                        count += 1;
                    }
                }
            } // completions borrow ends here

            // 2. PROCESS & RE-SUBMIT
            for i in 0..count {
                let (slot_idx, result) = completed_slots[i].unwrap();

                if result < 0 {
                    // Ignore transient errors
                    self.submit_recv(fd, journal, slot_idx, &mut in_flight_offsets)?;
                    continue;
                }

                let bytes_received = result as usize;
                let offset = in_flight_offsets[slot_idx];

                if bytes_received >= CausalEvent::size_bytes() {
                    let blob = journal.blob_storage();
                    let packet_data = &blob[offset..offset + bytes_received];

                    let event =
                        unsafe { std::ptr::read(packet_data.as_ptr() as *const CausalEvent) };

                    let payload = &packet_data[CausalEvent::size_bytes()..];
                    let mut hasher = Hasher::new();
                    hasher.update(payload);
                    let computed = hasher.finalize();

                    if computed == event.checksum {
                        let ts = LAMPORT_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
                        let sequenced_event = CausalEvent::new(
                            ts,
                            event.node_id,
                            event.stream_id,
                            offset as u64,
                            event.checksum,
                        );

                        if let Some(ring_slot) = cursor.advance_head() {
                            unsafe {
                                journal.write_event_at(ring_slot, &sequenced_event);
                            }
                            EVENTS_PROCESSED.fetch_add(1, AtomicOrdering::Relaxed);
                            BYTES_PROCESSED
                                .fetch_add(bytes_received as u64, AtomicOrdering::Relaxed);

                            // Real-time notification
                            if let Some(ipc) = &self.ipc {
                                // Send slot index (4 bytes)
                                ipc.broadcast(&ring_slot.to_le_bytes());
                            }
                        }
                    }
                }

                self.submit_recv(fd, journal, slot_idx, &mut in_flight_offsets)?;
            }
        }
    }

    /// Submits a new Recv request to io_uring, pointing directly into the next mmap chunk.
    fn submit_recv(
        &mut self,
        fd: types::Fd,
        journal: &mut Journal,
        slot_idx: usize,
        in_flight_offsets: &mut [usize],
    ) -> std::io::Result<()> {
        let offset = self.next_blob_offset;
        let blob = journal.blob_storage_mut();

        // Wrap blob offset if we're at the end (circular blob buffer)
        if offset + MAX_PACKET_SIZE > blob.len() {
            self.next_blob_offset = 0;
            return self.submit_recv(fd, journal, slot_idx, in_flight_offsets);
        }

        in_flight_offsets[slot_idx] = offset;
        self.next_blob_offset += MAX_PACKET_SIZE;

        // Pointer directly to the mmap'd region. Zero-copy!
        let buf_ptr = unsafe { blob.as_mut_ptr().add(offset) };

        let recv_entry = opcode::Recv::new(fd, buf_ptr, MAX_PACKET_SIZE as u32)
            .build()
            .user_data(slot_idx as u64);

        unsafe {
            self.ring
                .submission()
                .push(&recv_entry)
                .map_err(|_| std::io::Error::other("io_uring submission queue full"))?;
        }
        Ok(())
    }
}
