//! Streaming API implementation for large data handling
//! 
//! This module provides streaming capabilities for processing large data
//! that doesn't fit entirely in memory.

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::path::Path;

use async_trait::async_trait;
use futures::{Stream, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{Error, Result, SandboxError, ResourceKind};

/// Stream direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDirection {
    /// Host to guest (input to WebAssembly)
    HostToGuest,
    
    /// Guest to host (output from WebAssembly)
    GuestToHost,
    
    /// Bidirectional
    Bidirectional,
}

/// Stream type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamType {
    /// Raw bytes
    Bytes,
    
    /// UTF-8 text
    Text,
    
    /// JSON data
    Json,
    
    /// MessagePack data
    MessagePack,
}

/// Streaming channel configuration
#[derive(Debug, Clone)]
pub struct StreamingChannelConfig {
    /// Stream direction
    pub direction: StreamDirection,
    
    /// Stream type
    pub stream_type: StreamType,
    
    /// Buffer size for the channel
    pub buffer_size: usize,
    
    /// Maximum chunk size
    pub max_chunk_size: usize,
    
    /// Whether to validate UTF-8 for text streams
    pub validate_utf8: bool,
}

impl Default for StreamingChannelConfig {
    fn default() -> Self {
        Self {
            direction: StreamDirection::Bidirectional,
            stream_type: StreamType::Bytes,
            buffer_size: 64 * 1024, // 64KB
            max_chunk_size: 16 * 1024, // 16KB
            validate_utf8: true,
        }
    }
}

/// Streaming statistics
#[derive(Debug, Clone)]
pub struct StreamingStats {
    /// Number of bytes sent
    pub bytes_sent: u64,
    
    /// Number of bytes received
    pub bytes_received: u64,
    
    /// Number of chunks sent
    pub chunks_sent: u64,
    
    /// Number of chunks received
    pub chunks_received: u64,
    
    /// Average chunk size
    pub average_chunk_size: f64,
    
    /// Maximum chunk size seen
    pub max_chunk_size_seen: usize,
    
    /// Number of errors
    pub error_count: u64,
}

/// Streaming channel trait
pub trait StreamingChannel: Send + Sync {
    /// Get the channel ID
    fn id(&self) -> &str;
    
    /// Get the channel configuration
    fn config(&self) -> &StreamingChannelConfig;
    
    /// Get the streaming statistics
    fn stats(&self) -> StreamingStats;
    
    /// Check if the channel is open
    fn is_open(&self) -> bool;
    
    /// Close the channel
    fn close(&self) -> Result<()>;
}

/// Byte stream chunk with metadata
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Chunk data
    pub data: Vec<u8>,
    
    /// Whether this is the final chunk
    pub is_final: bool,
    
    /// Sequence number
    pub sequence: u64,
    
    /// Chunk metadata
    pub metadata: Option<serde_json::Value>,
}

/// Streaming input channel
#[async_trait]
pub trait StreamingInput: StreamingChannel {
    /// Send a chunk to the guest
    async fn send_chunk(&self, chunk: StreamChunk) -> Result<()>;
    
    /// Send raw bytes to the guest
    async fn send_bytes(&self, data: &[u8], is_final: bool) -> Result<()>;
    
    /// Send a stream of chunks to the guest
    async fn send_stream<S>(&self, stream: &mut S) -> Result<u64>
    where
        S: Stream<Item = Result<StreamChunk>> + Unpin + Send;
}

/// Streaming output channel
#[async_trait]
pub trait StreamingOutput: StreamingChannel {
    /// Receive a chunk from the guest
    async fn receive_chunk(&self) -> Result<StreamChunk>;
    
    /// Receive raw bytes from the guest
    async fn receive_bytes(&self, max_size: Option<usize>) -> Result<Vec<u8>>;
    
    /// Receive a stream of chunks from the guest
    fn receive_stream(&self) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;
}

/// Bidirectional streaming channel
pub trait StreamingChannel2Way: StreamingInput + StreamingOutput {}

/// Concrete implementation for dynamic dispatch
#[derive(Clone)]
pub enum StreamingChannel2WayImpl {
    Memory(MemoryStreamingChannel),
    File(FileStreamingChannel),
}

impl StreamingChannel for StreamingChannel2WayImpl {
    fn id(&self) -> &str {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.id(),
            StreamingChannel2WayImpl::File(ch) => ch.id(),
        }
    }
    
    fn config(&self) -> &StreamingChannelConfig {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.config(),
            StreamingChannel2WayImpl::File(ch) => ch.config(),
        }
    }
    
    fn stats(&self) -> StreamingStats {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.stats(),
            StreamingChannel2WayImpl::File(ch) => ch.stats(),
        }
    }
    
    fn is_open(&self) -> bool {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.is_open(),
            StreamingChannel2WayImpl::File(ch) => ch.is_open(),
        }
    }
    
    fn close(&self) -> Result<()> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.close(),
            StreamingChannel2WayImpl::File(ch) => ch.close(),
        }
    }
}

#[async_trait]
impl StreamingInput for StreamingChannel2WayImpl {
    async fn send_chunk(&self, chunk: StreamChunk) -> Result<()> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.send_chunk(chunk).await,
            StreamingChannel2WayImpl::File(ch) => ch.send_chunk(chunk).await,
        }
    }
    
    async fn send_bytes(&self, data: &[u8], is_final: bool) -> Result<()> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.send_bytes(data, is_final).await,
            StreamingChannel2WayImpl::File(ch) => ch.send_bytes(data, is_final).await,
        }
    }
    
    async fn send_stream<S>(&self, stream: &mut S) -> Result<u64>
    where
        S: Stream<Item = Result<StreamChunk>> + Unpin + Send
    {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.send_stream(stream).await,
            StreamingChannel2WayImpl::File(ch) => ch.send_stream(stream).await,
        }
    }
}

#[async_trait]
impl StreamingOutput for StreamingChannel2WayImpl {
    async fn receive_chunk(&self) -> Result<StreamChunk> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.receive_chunk().await,
            StreamingChannel2WayImpl::File(ch) => ch.receive_chunk().await,
        }
    }
    
    async fn receive_bytes(&self, max_size: Option<usize>) -> Result<Vec<u8>> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.receive_bytes(max_size).await,
            StreamingChannel2WayImpl::File(ch) => ch.receive_bytes(max_size).await,
        }
    }
    
    fn receive_stream(&self) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>> {
        match self {
            StreamingChannel2WayImpl::Memory(ch) => ch.receive_stream(),
            StreamingChannel2WayImpl::File(ch) => ch.receive_stream(),
        }
    }
}

impl StreamingChannel2Way for StreamingChannel2WayImpl {}

/// Concrete streaming channel types
#[derive(Clone)]
pub enum StreamingChannelType {
    Memory(Arc<MemoryStreamingChannel>),
    File(Arc<FileStreamingChannel>),
}

impl StreamingChannelType {
    pub fn id(&self) -> &str {
        match self {
            StreamingChannelType::Memory(channel) => &channel.id,
            StreamingChannelType::File(channel) => &channel.id,
        }
    }
    
    pub async fn send_chunk(&self, chunk: StreamChunk) -> Result<()> {
        match self {
            StreamingChannelType::Memory(channel) => channel.send_chunk(chunk).await,
            StreamingChannelType::File(channel) => channel.send_chunk(chunk).await,
        }
    }
    
    pub async fn receive_chunk(&self) -> Result<StreamChunk> {
        match self {
            StreamingChannelType::Memory(channel) => channel.receive_chunk().await,
            StreamingChannelType::File(channel) => channel.receive_chunk().await,
        }
    }
    
    pub async fn close(&self) -> Result<()> {
        match self {
            StreamingChannelType::Memory(channel) => channel.close(),
            StreamingChannelType::File(channel) => channel.close(),
        }
    }
}

/// Memory-based streaming channel implementation
#[derive(Clone)]
pub struct MemoryStreamingChannel {
    /// Channel ID
    id: String,
    
    /// Channel configuration
    config: StreamingChannelConfig,
    
    /// Internal state
    state: Arc<tokio::sync::RwLock<MemoryStreamingState>>,
}

/// Internal state for memory streaming channel
struct MemoryStreamingState {
    /// Whether the channel is open
    is_open: bool,
    
    /// Host to guest queue
    h2g_queue: Vec<StreamChunk>,
    
    /// Guest to host queue
    g2h_queue: Vec<StreamChunk>,
    
    /// Stream statistics
    stats: StreamingStats,
}

impl MemoryStreamingChannel {
    /// Create a new memory-based streaming channel
    pub fn new(id: impl Into<String>, config: StreamingChannelConfig) -> Self {
        Self {
            id: id.into(),
            config,
            state: Arc::new(tokio::sync::RwLock::new(MemoryStreamingState {
                is_open: true,
                h2g_queue: Vec::new(),
                g2h_queue: Vec::new(),
                stats: StreamingStats {
                    bytes_sent: 0,
                    bytes_received: 0,
                    chunks_sent: 0,
                    chunks_received: 0,
                    average_chunk_size: 0.0,
                    max_chunk_size_seen: 0,
                    error_count: 0,
                },
            })),
        }
    }
}

impl StreamingChannel for MemoryStreamingChannel {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn config(&self) -> &StreamingChannelConfig {
        &self.config
    }
    
    fn stats(&self) -> StreamingStats {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.state.read().await.stats.clone()
            })
        })
    }
    
    fn is_open(&self) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.state.read().await.is_open
            })
        })
    }
    
    fn close(&self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut state = self.state.write().await;
                state.is_open = false;
                Ok(())
            })
        })
    }
}

#[async_trait]
impl StreamingInput for MemoryStreamingChannel {
    async fn send_chunk(&self, chunk: StreamChunk) -> Result<()> {
        let mut state = self.state.write().await;
        
        if !state.is_open {
            return Err(Error::Communication {
                channel: "memory_streaming".to_string(),
                reason: "Streaming channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        if chunk.data.len() > self.config.max_chunk_size {
            state.stats.error_count += 1;
            return Err(Error::ResourceExhausted {
                kind: ResourceKind::Memory,
                limit: self.config.max_chunk_size as u64,
                used: chunk.data.len() as u64,
                instance_id: None,
                suggestion: Some(format!("Consider reducing chunk size to {} bytes or less", self.config.max_chunk_size)),
            });
        }
        
        // Update statistics
        state.stats.bytes_sent += chunk.data.len() as u64;
        state.stats.chunks_sent += 1;
        
        // Update average chunk size
        let new_avg = ((state.stats.average_chunk_size * (state.stats.chunks_sent - 1) as f64) 
            + chunk.data.len() as f64) / state.stats.chunks_sent as f64;
        state.stats.average_chunk_size = new_avg;
        
        // Update max chunk size
        state.stats.max_chunk_size_seen = state.stats.max_chunk_size_seen.max(chunk.data.len());
        
        // Add to queue
        state.h2g_queue.push(chunk);
        
        Ok(())
    }
    
    async fn send_bytes(&self, data: &[u8], is_final: bool) -> Result<()> {
        // Create a chunk from raw bytes
        let chunk = StreamChunk {
            data: data.to_vec(),
            is_final,
            sequence: 0, // Will be assigned in send_chunk
            metadata: None,
        };
        
        self.send_chunk(chunk).await
    }
    
    async fn send_stream<S>(&self, stream: &mut S) -> Result<u64>
    where
        S: Stream<Item = Result<StreamChunk>> + Unpin + Send,
    {
        let mut bytes_sent = 0;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            bytes_sent += chunk.data.len() as u64;
            self.send_chunk(chunk).await?;
        }
        
        Ok(bytes_sent)
    }
}

#[async_trait]
impl StreamingOutput for MemoryStreamingChannel {
    async fn receive_chunk(&self) -> Result<StreamChunk> {
        let mut state = self.state.write().await;
        
        if !state.is_open {
            return Err(Error::Communication {
                channel: "memory_streaming".to_string(),
                reason: "Streaming channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        if state.g2h_queue.is_empty() {
            return Err(Error::NotFound {
                resource_type: "stream_data".to_string(),
                identifier: "No data available in streaming channel".to_string(),
            });
        }
        
        // Get the first chunk
        let chunk = state.g2h_queue.remove(0);
        
        // Update statistics
        state.stats.bytes_received += chunk.data.len() as u64;
        state.stats.chunks_received += 1;
        
        Ok(chunk)
    }
    
    async fn receive_bytes(&self, max_size: Option<usize>) -> Result<Vec<u8>> {
        let chunk = self.receive_chunk().await?;
        
        // Check size limit if specified
        if let Some(max) = max_size {
            if chunk.data.len() > max {
                return Err(Error::ResourceExhausted {
                    kind: ResourceKind::Memory,
                    limit: max as u64,
                    used: chunk.data.len() as u64,
                    instance_id: None,
                    suggestion: Some(format!("Consider increasing max_size to {} bytes or more", chunk.data.len())),
                });
            }
        }
        
        Ok(chunk.data)
    }
    
    fn receive_stream(&self) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>> {
        // Create a clone of the state for the stream
        let state = self.state.clone();
        
        // Create a stream that yields chunks from the g2h_queue
        let stream = async_stream::stream! {
            let mut last_seen_index = 0;
            
            loop {
                // Get the current state
                let state_guard = state.read().await;
                
                // Check if closed
                if !state_guard.is_open {
                    break;
                }
                
                // Get all new chunks
                let new_chunks: Vec<StreamChunk> = state_guard.g2h_queue
                    .iter()
                    .skip(last_seen_index)
                    .cloned()
                    .collect();
                
                // Update last seen index
                last_seen_index = state_guard.g2h_queue.len();
                
                // Drop the guard
                drop(state_guard);
                
                // Yield all new chunks
                for chunk in new_chunks {
                    let is_final = chunk.is_final;
                    yield Ok(chunk);
                    
                    // If this is the final chunk, break
                    if is_final {
                        return;
                    }
                }
                
                // Wait a bit before polling again
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            
            // If we get here, the channel was closed
            yield Err(Error::Communication {
                channel: "memory_streaming".to_string(),
                reason: "Streaming channel was closed".to_string(),
                instance_id: None,
            });
        };
        
        Box::pin(stream)
    }
}

impl StreamingChannel2Way for MemoryStreamingChannel {}

/// File-based streaming channel
#[derive(Clone)]
pub struct FileStreamingChannel {
    /// Channel ID
    id: String,
    
    /// Channel configuration
    config: StreamingChannelConfig,
    
    /// Input file path
    input_path: Option<std::path::PathBuf>,
    
    /// Output file path
    output_path: Option<std::path::PathBuf>,
    
    /// Internal state
    state: Arc<tokio::sync::RwLock<FileStreamingState>>,
}

/// Internal state for file streaming channel
struct FileStreamingState {
    /// Whether the channel is open
    is_open: bool,
    
    /// Stream statistics
    stats: StreamingStats,
}

impl FileStreamingChannel {
    /// Create a new file-based streaming channel
    pub fn new(
        id: impl Into<String>,
        config: StreamingChannelConfig,
        input_path: Option<impl AsRef<Path>>,
        output_path: Option<impl AsRef<Path>>,
    ) -> Self {
        Self {
            id: id.into(),
            config,
            input_path: input_path.map(|p| p.as_ref().to_path_buf()),
            output_path: output_path.map(|p| p.as_ref().to_path_buf()),
            state: Arc::new(tokio::sync::RwLock::new(FileStreamingState {
                is_open: true,
                stats: StreamingStats {
                    bytes_sent: 0,
                    bytes_received: 0,
                    chunks_sent: 0,
                    chunks_received: 0,
                    average_chunk_size: 0.0,
                    max_chunk_size_seen: 0,
                    error_count: 0,
                },
            })),
        }
    }
}

impl StreamingChannel for FileStreamingChannel {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn config(&self) -> &StreamingChannelConfig {
        &self.config
    }
    
    fn stats(&self) -> StreamingStats {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.state.read().await.stats.clone()
            })
        })
    }
    
    fn is_open(&self) -> bool {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.state.read().await.is_open
            })
        })
    }
    
    fn close(&self) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut state = self.state.write().await;
                state.is_open = false;
                Ok(())
            })
        })
    }
}

#[async_trait]
impl StreamingInput for FileStreamingChannel {
    async fn send_chunk(&self, chunk: StreamChunk) -> Result<()> {
        let mut state = self.state.write().await;
        
        if !state.is_open {
            return Err(Error::Communication {
                channel: "file_streaming".to_string(),
                reason: "Streaming channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        let output_path = match &self.output_path {
            Some(path) => path,
            None => return Err(Error::Configuration {
                message: "No output file configured".to_string(),
                suggestion: Some("Configure an output file path using with_output_file()".to_string()),
                field: Some("output_path".to_string()),
            }),
        };
        
        // Open the file for appending
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(output_path)
            .await
            .map_err(|e| Error::Filesystem {
                operation: "open".to_string(),
                path: output_path.clone(),
                reason: format!("Failed to open output file: {}", e),
            })?;
        
        // Write the chunk size and data
        let size = chunk.data.len() as u32;
        let size_bytes = size.to_le_bytes();
        
        file.write_all(size_bytes.as_slice()).await
            .map_err(|e| Error::Filesystem {
                operation: "write".to_string(),
                path: output_path.clone(),
                reason: format!("Failed to write chunk size: {}", e),
            })?;
            
        file.write_all(&chunk.data).await
            .map_err(|e| Error::Filesystem {
                operation: "write".to_string(),
                path: output_path.clone(),
                reason: format!("Failed to write chunk data: {}", e),
            })?;
        
        // Write is_final flag
        let final_flag = if chunk.is_final { 1u8 } else { 0u8 };
        file.write_all([final_flag].as_slice()).await
            .map_err(|e| Error::Filesystem {
                operation: "write".to_string(),
                path: output_path.clone(),
                reason: format!("Failed to write final flag: {}", e),
            })?;
        
        // Update statistics
        state.stats.bytes_sent += chunk.data.len() as u64;
        state.stats.chunks_sent += 1;
        
        // Update average chunk size
        let new_avg = ((state.stats.average_chunk_size * (state.stats.chunks_sent - 1) as f64) 
            + chunk.data.len() as f64) / state.stats.chunks_sent as f64;
        state.stats.average_chunk_size = new_avg;
        
        // Update max chunk size
        state.stats.max_chunk_size_seen = state.stats.max_chunk_size_seen.max(chunk.data.len());
        
        Ok(())
    }
    
    async fn send_bytes(&self, data: &[u8], is_final: bool) -> Result<()> {
        // Create a chunk from raw bytes
        let chunk = StreamChunk {
            data: data.to_vec(),
            is_final,
            sequence: 0,
            metadata: None,
        };
        
        self.send_chunk(chunk).await
    }
    
    async fn send_stream<S>(&self, stream: &mut S) -> Result<u64>
    where
        S: Stream<Item = Result<StreamChunk>> + Unpin + Send,
    {
        let mut bytes_sent = 0;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            bytes_sent += chunk.data.len() as u64;
            self.send_chunk(chunk).await?;
        }
        
        Ok(bytes_sent)
    }
}

#[async_trait]
impl StreamingOutput for FileStreamingChannel {
    async fn receive_chunk(&self) -> Result<StreamChunk> {
        let mut state = self.state.write().await;
        
        if !state.is_open {
            return Err(Error::Communication {
                channel: "file_streaming".to_string(),
                reason: "Streaming channel is closed".to_string(),
                instance_id: None,
            });
        }
        
        let input_path = match &self.input_path {
            Some(path) => path,
            None => return Err(Error::Configuration {
                message: "No input file configured".to_string(),
                suggestion: Some("Configure an input file path using with_input_file()".to_string()),
                field: Some("input_path".to_string()),
            }),
        };
        
        // Check if file exists
        if !tokio::fs::try_exists(input_path).await.unwrap_or(false) {
            return Err(Error::NotFound {
                resource_type: "input_file".to_string(),
                identifier: "Input file does not exist".to_string(),
            });
        }
        
        // Open the file for reading
        let mut file = tokio::fs::File::open(input_path).await
            .map_err(|e| Error::Filesystem {
                operation: "open".to_string(),
                path: input_path.clone(),
                reason: format!("Failed to open input file: {}", e),
            })?;
        
        // Get file metadata
        let metadata = file.metadata().await
            .map_err(|e| Error::Filesystem {
                operation: "metadata".to_string(),
                path: input_path.clone(),
                reason: format!("Failed to get file metadata: {}", e),
            })?;
            
        // Check if file is empty
        if metadata.len() == 0 {
            return Err(Error::NotFound {
                resource_type: "stream_data".to_string(),
                identifier: "Input file is empty".to_string(),
            });
        }
        
        // Read the chunk size
        let mut size_bytes = [0u8; 4];
        file.read_exact(&mut size_bytes).await
            .map_err(|e| Error::Filesystem {
                operation: "read".to_string(),
                path: input_path.clone(),
                reason: format!("Failed to read chunk size: {}", e),
            })?;
            
        let size = u32::from_le_bytes(size_bytes) as usize;
        
        // Read the chunk data
        let mut data = vec![0u8; size];
        file.read_exact(&mut data).await
            .map_err(|e| Error::Filesystem {
                operation: "read".to_string(),
                path: input_path.clone(),
                reason: format!("Failed to read chunk data: {}", e),
            })?;
        
        // Read is_final flag
        let mut final_flag = [0u8; 1];
        file.read_exact(&mut final_flag).await
            .map_err(|e| Error::Filesystem {
                operation: "read".to_string(),
                path: input_path.clone(),
                reason: format!("Failed to read final flag: {}", e),
            })?;
            
        let is_final = final_flag[0] != 0;
        
        // Create the chunk
        let chunk = StreamChunk {
            data,
            is_final,
            sequence: state.stats.chunks_received,
            metadata: None,
        };
        
        // Update statistics
        state.stats.bytes_received += chunk.data.len() as u64;
        state.stats.chunks_received += 1;
        
        Ok(chunk)
    }
    
    async fn receive_bytes(&self, max_size: Option<usize>) -> Result<Vec<u8>> {
        let chunk = self.receive_chunk().await?;
        
        // Check size limit if specified
        if let Some(max) = max_size {
            if chunk.data.len() > max {
                return Err(Error::ResourceExhausted {
                    kind: ResourceKind::Memory,
                    limit: max as u64,
                    used: chunk.data.len() as u64,
                    instance_id: None,
                    suggestion: Some(format!("Consider increasing max_size to {} bytes or more", chunk.data.len())),
                });
            }
        }
        
        Ok(chunk.data)
    }
    
    fn receive_stream(&self) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>> {
        // Create clones for the stream
        let state = self.state.clone();
        let input_path = self.input_path.clone();
        
        // Create a stream that reads chunks from the file
        let stream = async_stream::stream! {
            // Check if input path is configured
            let input_path = match &input_path {
                Some(path) => path.clone(),
                None => {
                    yield Err(Error::Configuration {
                        message: "No input file configured".to_string(),
                        suggestion: Some("Configure an input file path using with_input_file()".to_string()),
                        field: Some("input_path".to_string()),
                    });
                    return;
                }
            };
            
            // Try to open the file
            let file = match tokio::fs::File::open(&input_path).await {
                Ok(file) => file,
                Err(e) => {
                    yield Err(Error::Filesystem {
                        operation: "open".to_string(),
                        path: input_path.clone(),
                        reason: format!("Failed to open input file: {}", e),
                    });
                    return;
                }
            };
            
            // Create a buffered reader
            let mut reader = tokio::io::BufReader::new(file);
            let mut sequence = 0;
            
            loop {
                // Check if channel is still open
                {
                    let state_guard = state.read().await;
                    if !state_guard.is_open {
                        yield Err(Error::Communication {
                            channel: "file_streaming".to_string(),
                            reason: "Streaming channel was closed".to_string(),
                            instance_id: None,
                        });
                        return;
                    }
                }
                
                // Read chunk size
                let mut size_bytes = [0u8; 4];
                match reader.read_exact(&mut size_bytes).await {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        // End of file, wait for more data
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    Err(e) => {
                        yield Err(Error::Filesystem {
                            operation: "read".to_string(),
                            path: input_path.clone(),
                            reason: format!("Failed to read chunk size: {}", e),
                        });
                        return;
                    }
                }
                
                let size = u32::from_le_bytes(size_bytes) as usize;
                
                // Read chunk data
                let mut data = vec![0u8; size];
                if let Err(e) = reader.read_exact(&mut data).await {
                    yield Err(Error::Filesystem {
                        operation: "read".to_string(),
                        path: input_path.clone(),
                        reason: format!("Failed to read chunk data: {}", e),
                    });
                    return;
                }
                
                // Read is_final flag
                let mut final_flag = [0u8; 1];
                if let Err(e) = reader.read_exact(&mut final_flag).await {
                    yield Err(Error::Filesystem {
                        operation: "read".to_string(),
                        path: input_path.clone(),
                        reason: format!("Failed to read final flag: {}", e),
                    });
                    return;
                }
                
                let is_final = final_flag[0] != 0;
                
                // Create the chunk
                let chunk = StreamChunk {
                    data,
                    is_final,
                    sequence,
                    metadata: None,
                };
                
                // Update statistics
                {
                    let mut state_guard = state.write().await;
                    state_guard.stats.bytes_received += chunk.data.len() as u64;
                    state_guard.stats.chunks_received += 1;
                }
                
                // Yield the chunk
                yield Ok(chunk);
                sequence += 1;
                
                // If this is the final chunk, break
                if is_final {
                    break;
                }
            }
        };
        
        Box::pin(stream)
    }
}

impl StreamingChannel2Way for FileStreamingChannel {}

/// Streaming factory
pub struct StreamingFactory;

impl StreamingFactory {
    /// Create a new memory-based streaming channel
    pub fn create_memory_channel(
        id: impl Into<String>,
        config: Option<StreamingChannelConfig>,
    ) -> StreamingChannelType {
        let config = config.unwrap_or_default();
        StreamingChannelType::Memory(Arc::new(MemoryStreamingChannel::new(id, config)))
    }
    
    /// Create a new file-based streaming channel
    pub fn create_file_channel(
        id: impl Into<String>,
        config: Option<StreamingChannelConfig>,
        input_path: Option<impl AsRef<Path>>,
        output_path: Option<impl AsRef<Path>>,
    ) -> StreamingChannelType {
        let config = config.unwrap_or_default();
        StreamingChannelType::File(Arc::new(FileStreamingChannel::new(id, config, input_path, output_path)))
    }
}

/// Host function wrapper for handling streams
pub struct StreamingFunction<T, R> {
    /// Function to process the stream
    processor: Box<dyn Fn(T, Arc<StreamingChannel2WayImpl>) -> Result<R> + Send + Sync>,
    
    /// Input stream
    input_stream: Arc<StreamingChannel2WayImpl>,
}

impl<T, R> StreamingFunction<T, R>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    R: serde::Serialize + Send + 'static,
{
    /// Create a new streaming function
    pub fn new<F>(processor: F, input_stream: Arc<StreamingChannel2WayImpl>) -> Self
    where
        F: Fn(T, Arc<StreamingChannel2WayImpl>) -> Result<R> + Send + Sync + 'static,
    {
        Self {
            processor: Box::new(processor),
            input_stream,
        }
    }
    
    /// Process a request
    pub fn process(&self, request: T) -> Result<R> {
        (self.processor)(request, self.input_stream.clone())
    }
}

/// Extension methods for streaming with WebAssembly instances
#[async_trait]
pub trait StreamingInstanceExt {
    /// Call a function with streaming data
    async fn call_streaming_function<Params, Return>(
        &self,
        function_name: &str,
        params: &Params,
        stream: Arc<StreamingChannel2WayImpl>,
    ) -> Result<Return>
    where
        Params: serde::Serialize + ?Sized + Send + Sync,
        Return: serde::de::DeserializeOwned + 'static;
}

/// Streaming manager for coordinating streams between host and guest
pub struct StreamingManager {
    /// Active streaming channels
    channels: tokio::sync::RwLock<std::collections::HashMap<String, StreamingChannelType>>,
}

impl StreamingManager {
    /// Create a new streaming manager
    pub fn new() -> Self {
        Self {
            channels: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
    
    /// Create a new streaming channel
    pub async fn create_channel(
        &self,
        id: Option<String>,
        config: Option<StreamingChannelConfig>,
    ) -> Result<StreamingChannelType> {
        let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let channel = StreamingFactory::create_memory_channel(id.clone(), config);
        
        // Register the channel
        let mut channels = self.channels.write().await;
        channels.insert(id, channel.clone());
        
        Ok(channel)
    }
    
    /// Get a streaming channel by ID
    pub async fn get_channel(&self, id: &str) -> Option<StreamingChannelType> {
        let channels = self.channels.read().await;
        channels.get(id).cloned()
    }
    
    /// Close a streaming channel
    pub async fn close_channel(&self, id: &str) -> Result<()> {
        let mut channels = self.channels.write().await;
        
        if let Some(channel) = channels.remove(id) {
            channel.close().await?;
            Ok(())
        } else {
            Err(SandboxError::NotFound { 
                resource_type: "streaming_channel".to_string(), 
                identifier: id.to_string() 
            })
        }
    }
    
    /// List all active streaming channels
    pub async fn list_channels(&self) -> Vec<String> {
        let channels = self.channels.read().await;
        channels.keys().cloned().collect()
    }
}

impl Default for StreamingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for transforming data in streams
pub struct StreamTransformer;

impl StreamTransformer {
    /// Transform a byte stream to a text stream
    pub fn bytes_to_text_stream<S>(
        stream: S,
    ) -> impl Stream<Item = Result<String>>
    where
        S: Stream<Item = Result<StreamChunk>> + Send,
    {
        stream.map(|chunk_result| {
            let chunk = chunk_result?;
            let text = String::from_utf8(chunk.data)
                .map_err(|e| SandboxError::Serialization {
                    format: "utf-8".to_string(),
                    operation: "decode".to_string(),
                    reason: e.to_string(),
                })?;
            Ok(text)
        })
    }
    
    /// Transform a byte stream to a JSON stream
    pub fn bytes_to_json_stream<S, T>(
        stream: S,
    ) -> impl Stream<Item = Result<T>>
    where
        S: Stream<Item = Result<StreamChunk>> + Send,
        T: serde::de::DeserializeOwned,
    {
        stream.map(|chunk_result| {
            let chunk = chunk_result?;
            let value = serde_json::from_slice(&chunk.data)
                .map_err(|e| SandboxError::Serialization {
                    format: "json".to_string(),
                    operation: "decode".to_string(),
                    reason: e.to_string(),
                })?;
            Ok(value)
        })
    }
    
    /// Transform objects to a JSON byte stream
    pub fn objects_to_json_byte_stream<S, T>(
        stream: S,
    ) -> impl Stream<Item = Result<StreamChunk>>
    where
        S: Stream<Item = Result<T>> + Send,
        T: serde::Serialize,
    {
        stream.map(|item_result| {
            let item = item_result?;
            let json = serde_json::to_vec(&item)
                .map_err(|e| SandboxError::Serialization {
                    format: "json".to_string(),
                    operation: "encode".to_string(),
                    reason: e.to_string(),
                })?;
                
            Ok(StreamChunk {
                data: json,
                is_final: false, // Set to true for last chunk
                sequence: 0,     // Will be assigned by the channel
                metadata: None,
            })
        })
    }
}
