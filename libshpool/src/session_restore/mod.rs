// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::VecDeque;

use shpool_protocol::TtySize;
use tracing::info;
use anyhow::{anyhow, Result};


/// Parse memory size string like "5MB", "1GB", "512KB" to bytes
fn parse_memory_size(size_str: &str) -> Result<usize> {
    if size_str == "0" {
        return Ok(0);
    }
    
    let size_str = size_str.trim().to_uppercase();
    
    if size_str.ends_with("KB") {
        let num_str = &size_str[..size_str.len() - 2];
        let num: usize = num_str.parse()?;
        Ok(num * 1024)
    } else if size_str.ends_with("MB") {
        let num_str = &size_str[..size_str.len() - 2];
        let num: usize = num_str.parse()?;
        Ok(num * 1024 * 1024)
    } else if size_str.ends_with("GB") {
        let num_str = &size_str[..size_str.len() - 2];
        let num: usize = num_str.parse()?;
        Ok(num * 1024 * 1024 * 1024)
    } else {
        Err(anyhow!("Invalid memory size format: {}. Use formats like '5MB', '1GB', '512KB', or '0'", size_str))
    }
}


pub trait SessionSpool {
    /// Resizes the internal representation to new tty size.
    fn resize(&mut self, size: TtySize);

    /// Gets a byte sequence to restore the on-screen session content.
    ///
    /// The returned sequence is expected to be able to restore the screen
    /// content regardless of any prior screen state. It thus mostly likely
    /// includes some terminal control codes to reset the screen from any
    /// state back to a known good state.
    ///
    /// Note that what exactly is restored is determined by the implementation,
    /// and thus can vary from do nothing to a few lines to a full screen,
    /// etc.
    fn restore_buffer(&self) -> Vec<u8>;

    /// Process bytes from pty master.
    fn process(&mut self, bytes: &[u8]);
}

/// A spool that only sends SIGWINCH signals, no caching.
pub struct SignalOnlySpool;
impl SessionSpool for SignalOnlySpool {
    fn resize(&mut self, _: TtySize) {}

    fn restore_buffer(&self) -> Vec<u8> {
        vec![]
    }

    fn process(&mut self, _: &[u8]) {}
}

/// A memory-based spool that keeps a fixed-size buffer of terminal output.
pub struct MemorySpool {
    buffer: VecDeque<u8>,
    max_size: usize,
    current_size: usize,
}

impl MemorySpool {
    fn new(max_size: usize) -> Self {
        MemorySpool {
            buffer: VecDeque::new(),
            max_size,
            current_size: 0,
        }
    }
}

impl SessionSpool for MemorySpool {
    fn resize(&mut self, _size: TtySize) {
        // Memory-based spool doesn't need to handle resize
    }

    fn restore_buffer(&self) -> Vec<u8> {
        let restore_buf: Vec<u8> = self.buffer.iter().copied().collect();
        info!("computing memory restore buf with {} bytes", restore_buf.len());
        if restore_buf.is_empty() {
            info!("restore buffer is empty - no content to restore");
        } else {
            info!("restore buffer content preview: {:?}", 
                  String::from_utf8_lossy(&restore_buf[..std::cmp::min(100, restore_buf.len())]));
        }
        restore_buf
    }

    fn process(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        
        info!("MemorySpool processing {} bytes", bytes.len());
        
        // Add new bytes to the buffer
        for &byte in bytes {
            self.buffer.push_back(byte);
            self.current_size += 1;
        }
        
        // Trim buffer if it exceeds max size
        while self.current_size > self.max_size && !self.buffer.is_empty() {
            self.buffer.pop_front();
            self.current_size -= 1;
        }
    }
}



/// Creates a spool given a memory size string like "5MB", "1MB", or "0".
pub fn new(
    restore_config: &str,
    _size: &TtySize,
) -> Result<Box<dyn SessionSpool + 'static>> {
    match parse_memory_size(restore_config) {
        Ok(0) => {
            info!("Creating SignalOnlySpool (no caching, SIGWINCH only)");
            Ok(Box::new(SignalOnlySpool))
        },
        Ok(max_size) => {
            info!("Creating MemorySpool with {} bytes limit", max_size);
            Ok(Box::new(MemorySpool::new(max_size)))
        },
        Err(e) => {
            Err(anyhow!("Failed to parse session_restore config '{}': {}", restore_config, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_memory_size() {
        // Test basic parsing
        assert_eq!(parse_memory_size("0").unwrap(), 0);
        assert_eq!(parse_memory_size("512KB").unwrap(), 512 * 1024);
        assert_eq!(parse_memory_size("5MB").unwrap(), 5 * 1024 * 1024);
        assert_eq!(parse_memory_size("2GB").unwrap(), 2 * 1024 * 1024 * 1024);

        // Test case insensitivity
        assert_eq!(parse_memory_size("1mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_size("1Mb").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_size("1kb").unwrap(), 1024);

        // Test with whitespace
        assert_eq!(parse_memory_size("  5MB  ").unwrap(), 5 * 1024 * 1024);

        // Test error cases
        assert!(parse_memory_size("invalid").is_err());
        assert!(parse_memory_size("5TB").is_err());
        assert!(parse_memory_size("5").is_err());
        assert!(parse_memory_size("MB").is_err());
        assert!(parse_memory_size("").is_err());
    }

    #[test]
    fn test_new_session_spool() {
        let tty_size = TtySize { rows: 24, cols: 80, xpixel: 0, ypixel: 0 };

        // Test creating SignalOnlySpool
        let spool = new("0", &tty_size).unwrap();
        assert_eq!(spool.restore_buffer().len(), 0);

        // Test creating MemorySpool
        let spool = new("1MB", &tty_size).unwrap();
        assert_eq!(spool.restore_buffer().len(), 0); // Initially empty

        // Test error case
        assert!(new("invalid", &tty_size).is_err());
    }

    #[test]
    fn test_memory_spool_functionality() {
        let mut spool = MemorySpool::new(100); // Small size for testing
        
        // Test initial state
        assert_eq!(spool.restore_buffer().len(), 0);
        
        // Add some data
        spool.process(b"hello");
        let buffer = spool.restore_buffer();
        assert_eq!(buffer, b"hello");
        
        // Add more data that exceeds capacity
        let large_data = vec![b'x'; 200];
        spool.process(&large_data);
        let buffer = spool.restore_buffer();
        
        // Should be trimmed to max_size
        assert!(buffer.len() <= 100);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_signal_only_spool() {
        let mut spool = SignalOnlySpool;
        
        // Should always return empty buffer
        assert_eq!(spool.restore_buffer().len(), 0);
        
        spool.process(b"some data");
        assert_eq!(spool.restore_buffer().len(), 0);
        
        spool.resize(TtySize { rows: 50, cols: 100, xpixel: 0, ypixel: 0 });
        assert_eq!(spool.restore_buffer().len(), 0);
    }
}
