//! I/O redirection for sandboxed processes

use std::sync::{Arc, Mutex};
use std::io::{self, Read, Write};

use crate::error::Result;
use crate::communication::CommunicationChannel;

/// Standard input/output redirection
pub struct StdioRedirection {
    /// Standard input channel
    stdin_channel: Arc<dyn CommunicationChannel>,
    
    /// Standard output channel
    stdout_channel: Arc<dyn CommunicationChannel>,
    
    /// Standard error channel
    stderr_channel: Arc<dyn CommunicationChannel>,
    
    /// Buffered stdin data
    stdin_buffer: Mutex<Vec<u8>>,
    
    /// Is closed
    closed: Mutex<bool>,
}

impl StdioRedirection {
    /// Create a new stdio redirection
    pub fn new(
        stdin_channel: Arc<dyn CommunicationChannel>,
        stdout_channel: Arc<dyn CommunicationChannel>,
        stderr_channel: Arc<dyn CommunicationChannel>,
    ) -> Self {
        Self {
            stdin_channel,
            stdout_channel,
            stderr_channel,
            stdin_buffer: Mutex::new(Vec::new()),
            closed: Mutex::new(false),
        }
    }
    
    /// Write to stdout
    pub fn write_stdout(&self, data: &[u8]) -> Result<()> {
        self.stdout_channel.send_to_guest(data)
    }
    
    /// Write to stderr
    pub fn write_stderr(&self, data: &[u8]) -> Result<()> {
        self.stderr_channel.send_to_guest(data)
    }
    
    /// Read from stdin
    pub fn read_stdin(&self, buf: &mut [u8]) -> Result<usize> {
        // If closed, return EOF
        if *self.closed.lock().unwrap() {
            return Ok(0);
        }
        
        // Try to fill the buffer if it's empty
        let mut stdin_buffer = self.stdin_buffer.lock().unwrap();
        if stdin_buffer.is_empty() {
            if let Ok(data) = self.stdin_channel.receive_from_guest() {
                stdin_buffer.extend_from_slice(&data);
            }
        }
        
        // If we have data, copy it to the buffer
        if !stdin_buffer.is_empty() {
            let n = std::cmp::min(buf.len(), stdin_buffer.len());
            buf[..n].copy_from_slice(&stdin_buffer[..n]);
            stdin_buffer.drain(..n);
            Ok(n)
        } else {
            // No data available
            Ok(0)
        }
    }
    
    /// Close the redirection
    pub fn close(&self) -> Result<()> {
        let mut closed = self.closed.lock().unwrap();
        *closed = true;
        
        self.stdin_channel.close()?;
        self.stdout_channel.close()?;
        self.stderr_channel.close()?;
        
        Ok(())
    }
}

/// Standard input implementation
pub struct StdioInput {
    /// Redirection
    redirection: Arc<StdioRedirection>,
}

impl StdioInput {
    /// Create a new stdin implementation
    pub fn new(redirection: Arc<StdioRedirection>) -> Self {
        Self {
            redirection,
        }
    }
}

impl Read for StdioInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.redirection.read_stdin(buf)
            .map_err(|e| io::Error::other(format!("{:?}", e)))
    }
}

/// Standard output implementation
pub struct StdioOutput {
    /// Redirection
    redirection: Arc<StdioRedirection>,
    
    /// Whether this is stderr
    is_stderr: bool,
}

impl StdioOutput {
    /// Create a new stdout implementation
    pub fn new_stdout(redirection: Arc<StdioRedirection>) -> Self {
        Self {
            redirection,
            is_stderr: false,
        }
    }
    
    /// Create a new stderr implementation
    pub fn new_stderr(redirection: Arc<StdioRedirection>) -> Self {
        Self {
            redirection,
            is_stderr: true,
        }
    }
}

impl Write for StdioOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = if self.is_stderr {
            self.redirection.write_stderr(buf)
        } else {
            self.redirection.write_stdout(buf)
        };
        
        match result {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(io::Error::other(format!("{:?}", e))),
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Factory for creating stdio redirections
pub struct StdioFactory {
    /// Channel factory
    channel_factory: Arc<dyn crate::communication::CommunicationFactory>,
}

impl StdioFactory {
    /// Create a new stdio factory
    pub fn new(channel_factory: Arc<dyn crate::communication::CommunicationFactory>) -> Self {
        Self {
            channel_factory,
        }
    }
    
    /// Create a new stdio redirection
    pub fn create_redirection(&self) -> Result<Arc<StdioRedirection>> {
        // Create the channels
        let stdin_channel = self.channel_factory.create_channel()?;
        let stdout_channel = self.channel_factory.create_channel()?;
        let stderr_channel = self.channel_factory.create_channel()?;
        
        // Create the redirection
        let redirection = StdioRedirection::new(stdin_channel, stdout_channel, stderr_channel);
        
        Ok(Arc::new(redirection))
    }
}
