//! Ring buffer consumer for reading rkBPF events.
//!
//! This module provides a userspace consumer for BPF ring buffers,
//! allowing efficient reading of kernel events via memory mapping.

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Ring buffer header structure (matches kernel definition).
#[repr(C)]
struct RingBufHeader {
    /// Consumer position (userspace updates this)
    consumer_pos: AtomicU64,
    /// Padding to cache line
    _pad1: [u8; 56],
    /// Producer position (kernel updates this)
    producer_pos: AtomicU64,
    /// Padding to cache line
    _pad2: [u8; 56],
}

/// Ring buffer record header.
#[repr(C)]
#[derive(Clone, Copy)]
struct RecordHeader {
    /// Length of the record data
    len: u32,
    /// Page offset (internal use)
    pg_off: u32,
}

impl RecordHeader {
    /// Flag indicating record is busy (being written)
    const BPF_RINGBUF_BUSY_BIT: u32 = 1 << 31;
    /// Flag indicating record should be discarded
    const BPF_RINGBUF_DISCARD_BIT: u32 = 1 << 30;

    /// Check if the record is busy (still being written).
    fn is_busy(&self) -> bool {
        self.len & Self::BPF_RINGBUF_BUSY_BIT != 0
    }

    /// Check if the record should be discarded.
    fn is_discarded(&self) -> bool {
        self.len & Self::BPF_RINGBUF_DISCARD_BIT != 0
    }

    /// Get the actual data length.
    fn data_len(&self) -> u32 {
        self.len & !(Self::BPF_RINGBUF_BUSY_BIT | Self::BPF_RINGBUF_DISCARD_BIT)
    }
}

/// Errors that can occur when working with ring buffers.
#[derive(Debug, thiserror::Error)]
pub enum RingBufError {
    /// Failed to open the ring buffer file
    #[error("failed to open ring buffer: {0}")]
    Open(std::io::Error),

    /// Failed to memory map the ring buffer
    #[error("failed to mmap ring buffer: {0}")]
    Mmap(std::io::Error),

    /// Invalid ring buffer size
    #[error("invalid ring buffer size: {0}")]
    InvalidSize(usize),

    /// Ring buffer path not found
    #[error("ring buffer not found: {0}")]
    NotFound(String),
}

/// Consumer for reading events from a BPF ring buffer.
pub struct RingBufConsumer {
    /// Memory-mapped header region
    header: *mut RingBufHeader,
    /// Memory-mapped data region
    data: *const u8,
    /// Size of the data region (power of 2)
    data_size: usize,
    /// Mask for wrapping (data_size - 1)
    mask: usize,
    /// File descriptor (kept open for the mapping lifetime)
    _file: File,
}

// Safety: The ring buffer is thread-safe through atomic operations
unsafe impl Send for RingBufConsumer {}
unsafe impl Sync for RingBufConsumer {}

impl RingBufConsumer {
    /// Open a ring buffer from a BPF filesystem path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the pinned ring buffer map (e.g., `/sys/fs/bpf/maps/events`)
    ///
    /// # Errors
    ///
    /// Returns an error if the path doesn't exist or the mmap fails.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, RingBufError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(RingBufError::NotFound(path.display().to_string()));
        }

        let file = File::open(path).map_err(RingBufError::Open)?;
        let fd = file.as_raw_fd();

        // Get the map info to determine size
        // For now, we'll use a reasonable default and let the kernel tell us
        let data_size = Self::get_ringbuf_size(fd)?;

        if !data_size.is_power_of_two() {
            return Err(RingBufError::InvalidSize(data_size));
        }

        let header_size = std::mem::size_of::<RingBufHeader>();
        let total_size = header_size + data_size;

        // Memory map the ring buffer
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                total_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(RingBufError::Mmap(std::io::Error::last_os_error()));
        }

        let header = ptr as *mut RingBufHeader;
        let data = unsafe { (ptr as *const u8).add(header_size) };

        Ok(Self {
            header,
            data,
            data_size,
            mask: data_size - 1,
            _file: file,
        })
    }

    /// Get the ring buffer size from the kernel.
    fn get_ringbuf_size(fd: i32) -> Result<usize, RingBufError> {
        // Use BPF_OBJ_GET_INFO_BY_FD to get the map info
        // For simplicity, we'll use a default size that can be overridden
        // In production, this would query the kernel for the actual size
        let _ = fd;

        // Default to 64KB - this matches common rkBPF ring buffer sizes
        // The actual size should be queried from the kernel
        Ok(64 * 1024)
    }

    /// Poll for available events without blocking.
    ///
    /// Returns an iterator over available events.
    pub fn poll(&self) -> RingBufIter<'_> {
        RingBufIter { consumer: self }
    }

    /// Read the next event from the ring buffer.
    ///
    /// Returns `None` if no events are available.
    pub fn read_event(&self) -> Option<Vec<u8>> {
        let header = unsafe { &*self.header };

        let cons_pos = header.consumer_pos.load(Ordering::Acquire);
        let prod_pos = header.producer_pos.load(Ordering::Acquire);

        if cons_pos >= prod_pos {
            return None;
        }

        // Read the record header
        let record_offset = (cons_pos as usize) & self.mask;
        let record_header = unsafe {
            let ptr = self.data.add(record_offset) as *const RecordHeader;
            *ptr
        };

        // Check if the record is still being written
        if record_header.is_busy() {
            return None;
        }

        let data_len = record_header.data_len() as usize;
        let header_size = std::mem::size_of::<RecordHeader>();

        // Calculate total record size (header + data, 8-byte aligned)
        let record_size = (header_size + data_len + 7) & !7;

        // Read the data if not discarded
        let data = if record_header.is_discarded() {
            Vec::new()
        } else {
            let data_offset = (record_offset + header_size) & self.mask;
            let mut data = vec![0u8; data_len];

            // Handle wrap-around
            let first_chunk = std::cmp::min(data_len, self.data_size - data_offset);
            unsafe {
                std::ptr::copy_nonoverlapping(self.data.add(data_offset), data.as_mut_ptr(), first_chunk);

                if first_chunk < data_len {
                    std::ptr::copy_nonoverlapping(
                        self.data,
                        data.as_mut_ptr().add(first_chunk),
                        data_len - first_chunk,
                    );
                }
            }

            data
        };

        // Advance consumer position
        let new_cons_pos = cons_pos + record_size as u64;
        header.consumer_pos.store(new_cons_pos, Ordering::Release);

        if data.is_empty() {
            // Discarded record, try next
            self.read_event()
        } else {
            Some(data)
        }
    }

    /// Get the number of bytes available to read.
    pub fn available(&self) -> usize {
        let header = unsafe { &*self.header };
        let cons_pos = header.consumer_pos.load(Ordering::Relaxed);
        let prod_pos = header.producer_pos.load(Ordering::Relaxed);

        (prod_pos.saturating_sub(cons_pos)) as usize
    }

    /// Check if the ring buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.available() == 0
    }
}

impl Drop for RingBufConsumer {
    fn drop(&mut self) {
        let header_size = std::mem::size_of::<RingBufHeader>();
        let total_size = header_size + self.data_size;

        unsafe {
            libc::munmap(self.header as *mut libc::c_void, total_size);
        }
    }
}

/// Iterator over ring buffer events.
pub struct RingBufIter<'a> {
    consumer: &'a RingBufConsumer,
}

impl Iterator for RingBufIter<'_> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consumer.read_event()
    }
}

/// Mock ring buffer for testing without kernel interaction.
pub struct MockRingBuf {
    events: Vec<Vec<u8>>,
    position: usize,
}

impl MockRingBuf {
    /// Create a new mock ring buffer with pre-loaded events.
    pub fn new(events: Vec<Vec<u8>>) -> Self {
        Self { events, position: 0 }
    }

    /// Read the next event.
    pub fn read_event(&mut self) -> Option<Vec<u8>> {
        if self.position < self.events.len() {
            let event = self.events[self.position].clone();
            self.position += 1;
            Some(event)
        } else {
            None
        }
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.position >= self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_header_flags() {
        let busy = RecordHeader {
            len: RecordHeader::BPF_RINGBUF_BUSY_BIT | 100,
            pg_off: 0,
        };
        assert!(busy.is_busy());
        assert!(!busy.is_discarded());
        assert_eq!(busy.data_len(), 100);

        let discarded = RecordHeader {
            len: RecordHeader::BPF_RINGBUF_DISCARD_BIT | 50,
            pg_off: 0,
        };
        assert!(!discarded.is_busy());
        assert!(discarded.is_discarded());
        assert_eq!(discarded.data_len(), 50);
    }

    #[test]
    fn test_mock_ringbuf() {
        let events = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

        let mut mock = MockRingBuf::new(events);
        assert!(!mock.is_empty());

        assert_eq!(mock.read_event(), Some(vec![1, 2, 3]));
        assert_eq!(mock.read_event(), Some(vec![4, 5, 6]));
        assert_eq!(mock.read_event(), Some(vec![7, 8, 9]));
        assert_eq!(mock.read_event(), None);
        assert!(mock.is_empty());
    }
}
