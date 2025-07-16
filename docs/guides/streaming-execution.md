# Streaming Execution Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Handle large data processing with streaming patterns, memory-efficient techniques, and backpressure management in wasm-sandbox.

## Overview

Streaming execution enables:

- **Memory Efficiency** - Process data larger than available memory
- **Low Latency** - Start processing before all data arrives
- **Backpressure** - Handle flow control automatically
- **Fault Tolerance** - Resume processing from checkpoints

## Quick Start

```rust
use wasm_sandbox::{StreamingSandbox, StreamConfig};
use tokio_stream::{Stream, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = StreamingSandbox::builder()
        .buffer_size(64 * 1024)  // 64KB buffer
        .max_concurrency(4)
        .enable_backpressure(true)
        .build()
        .await?;

    // Create streaming processor
    let mut processor = sandbox.create_stream_processor("data_transformer.wasm").await?;

    // Process data stream
    let input_stream = create_data_stream();
    let output_stream = processor.process_stream(input_stream).await?;

    // Consume results
    tokio::pin!(output_stream);
    while let Some(result) = output_stream.next().await {
        match result {
            Ok(data) => println!("Processed: {:?}", data),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}

async fn create_data_stream() -> impl Stream<Item = Vec<u8>> {
    tokio_stream::iter(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ])
}
```

## Stream Processing Architecture

```rust
pub struct StreamProcessor {
    sandbox: WasmSandbox,
    config: StreamConfig,
    buffer_manager: BufferManager,
    flow_controller: FlowController,
    checkpointer: Checkpointer,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub max_concurrency: usize,
    pub enable_backpressure: bool,
    pub checkpoint_interval: Duration,
    pub recovery_strategy: RecoveryStrategy,
    pub flow_control: FlowControlConfig,
}

impl StreamProcessor {
    pub async fn process_stream<I, O>(&mut self, input: I) -> Result<impl Stream<Item = Result<O, StreamError>>, StreamError>
    where
        I: Stream<Item = Vec<u8>> + Send + 'static,
        O: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(self.config.buffer_size);
        
        // Start processing task
        let processor_handle = self.start_processing_task(input, tx).await?;
        
        // Return output stream
        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn start_processing_task<I>(&self, mut input: I, output: tokio::sync::mpsc::Sender<Result<Vec<u8>, StreamError>>) -> Result<tokio::task::JoinHandle<()>, StreamError>
    where
        I: Stream<Item = Vec<u8>> + Send + 'static,
    {
        let sandbox = self.sandbox.clone();
        let config = self.config.clone();
        let mut checkpointer = self.checkpointer.clone();
        let flow_controller = self.flow_controller.clone();

        let handle = tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut chunk_count = 0u64;

            tokio::pin!(input);
            
            while let Some(chunk) = input.next().await {
                // Check backpressure
                if config.enable_backpressure {
                    flow_controller.wait_for_capacity().await;
                }

                buffer.extend_from_slice(&chunk);

                // Process when buffer is full or stream ends
                if buffer.len() >= config.buffer_size {
                    match Self::process_chunk(&sandbox, &buffer).await {
                        Ok(result) => {
                            if output.send(Ok(result)).await.is_err() {
                                break; // Receiver dropped
                            }
                        }
                        Err(e) => {
                            if output.send(Err(e)).await.is_err() {
                                break;
                            }
                        }
                    }

                    buffer.clear();
                    chunk_count += 1;

                    // Checkpoint periodically
                    if chunk_count % 100 == 0 {
                        if let Err(e) = checkpointer.save_checkpoint(chunk_count).await {
                            eprintln!("Checkpoint save failed: {}", e);
                        }
                    }
                }
            }

            // Process remaining data
            if !buffer.is_empty() {
                if let Ok(result) = Self::process_chunk(&sandbox, &buffer).await {
                    let _ = output.send(Ok(result)).await;
                }
            }
        });

        Ok(handle)
    }

    async fn process_chunk(sandbox: &WasmSandbox, data: &[u8]) -> Result<Vec<u8>, StreamError> {
        sandbox.call("process_chunk", data).await
            .map_err(StreamError::ProcessingFailed)
    }
}
```

## Memory Management

```rust
pub struct BufferManager {
    pools: Vec<MemoryPool>,
    current_pool: usize,
    allocation_strategy: AllocationStrategy,
}

impl BufferManager {
    pub fn new(config: &BufferConfig) -> Self {
        let mut pools = Vec::new();
        
        // Create multiple memory pools for different sizes
        pools.push(MemoryPool::new(1024, 100));      // 1KB buffers
        pools.push(MemoryPool::new(64 * 1024, 50));  // 64KB buffers
        pools.push(MemoryPool::new(1024 * 1024, 10)); // 1MB buffers

        Self {
            pools,
            current_pool: 0,
            allocation_strategy: config.allocation_strategy,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Result<Buffer, AllocationError> {
        // Find appropriate pool
        let pool_index = self.find_suitable_pool(size)?;
        
        // Allocate from pool
        self.pools[pool_index].allocate()
            .ok_or(AllocationError::PoolExhausted)
    }

    pub fn deallocate(&mut self, buffer: Buffer) {
        if let Some(pool) = self.pools.get_mut(buffer.pool_index) {
            pool.deallocate(buffer);
        }
    }
}

pub struct MemoryPool {
    buffer_size: usize,
    available: Vec<Buffer>,
    total_allocated: usize,
    max_buffers: usize,
}

impl MemoryPool {
    pub fn allocate(&mut self) -> Option<Buffer> {
        if let Some(buffer) = self.available.pop() {
            Some(buffer)
        } else if self.total_allocated < self.max_buffers {
            // Allocate new buffer
            let data = vec![0u8; self.buffer_size];
            self.total_allocated += 1;
            Some(Buffer {
                data: data.into_boxed_slice(),
                pool_index: 0, // Set by BufferManager
            })
        } else {
            None
        }
    }
}
```

## Flow Control and Backpressure

```rust
pub struct FlowController {
    capacity: Arc<Semaphore>,
    metrics: FlowMetrics,
    config: FlowControlConfig,
}

#[derive(Debug, Clone)]
pub struct FlowControlConfig {
    pub max_in_flight: usize,
    pub low_watermark: usize,
    pub high_watermark: usize,
    pub backpressure_strategy: BackpressureStrategy,
}

#[derive(Debug, Clone)]
pub enum BackpressureStrategy {
    Block,
    Drop,
    Sample,
    Adaptive,
}

impl FlowController {
    pub async fn wait_for_capacity(&self) {
        match self.config.backpressure_strategy {
            BackpressureStrategy::Block => {
                let _permit = self.capacity.acquire().await.unwrap();
                // Permit is automatically released when dropped
            }
            BackpressureStrategy::Drop => {
                if self.capacity.available_permits() == 0 {
                    self.metrics.record_drop();
                    return;
                }
            }
            BackpressureStrategy::Adaptive => {
                self.adaptive_backpressure().await;
            }
            _ => {}
        }
    }

    async fn adaptive_backpressure(&self) {
        let current_load = self.calculate_current_load();
        
        if current_load > 0.8 {
            // High load - increase backpressure
            tokio::time::sleep(Duration::from_millis(10)).await;
        } else if current_load < 0.3 {
            // Low load - reduce backpressure
            // Continue without delay
        } else {
            // Medium load - moderate backpressure
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
}
```

## Checkpointing and Recovery

```rust
pub struct Checkpointer {
    storage: CheckpointStorage,
    interval: Duration,
    last_checkpoint: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub position: StreamPosition,
    pub state: ProcessorState,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Checkpointer {
    pub async fn save_checkpoint(&mut self, position: u64) -> Result<(), CheckpointError> {
        let now = Instant::now();
        
        if let Some(last) = self.last_checkpoint {
            if now.duration_since(last) < self.interval {
                return Ok(()); // Too soon for next checkpoint
            }
        }

        let checkpoint = Checkpoint {
            timestamp: chrono::Utc::now(),
            position: StreamPosition::Offset(position),
            state: self.capture_processor_state().await?,
            metadata: HashMap::new(),
        };

        self.storage.save(checkpoint).await?;
        self.last_checkpoint = Some(now);
        
        Ok(())
    }

    pub async fn restore_from_checkpoint(&self) -> Result<Option<Checkpoint>, CheckpointError> {
        self.storage.load_latest().await
    }
}

pub trait CheckpointStorage: Send + Sync {
    async fn save(&self, checkpoint: Checkpoint) -> Result<(), CheckpointError>;
    async fn load_latest(&self) -> Result<Option<Checkpoint>, CheckpointError>;
    async fn list_checkpoints(&self) -> Result<Vec<CheckpointInfo>, CheckpointError>;
}

pub struct FileCheckpointStorage {
    directory: PathBuf,
}

impl CheckpointStorage for FileCheckpointStorage {
    async fn save(&self, checkpoint: Checkpoint) -> Result<(), CheckpointError> {
        let filename = format!("checkpoint_{}.json", checkpoint.timestamp.timestamp());
        let path = self.directory.join(filename);
        
        let data = serde_json::to_vec_pretty(&checkpoint)?;
        tokio::fs::write(path, data).await?;
        
        Ok(())
    }
}
```

## Performance Optimization

```rust
pub struct StreamOptimizer {
    profiler: StreamProfiler,
    adaptive_config: AdaptiveConfig,
}

impl StreamOptimizer {
    pub async fn optimize_stream(&mut self, processor: &mut StreamProcessor) -> Result<(), OptimizationError> {
        let metrics = self.profiler.collect_metrics().await;
        
        // Analyze bottlenecks
        if metrics.memory_pressure > 0.8 {
            self.optimize_memory_usage(processor).await?;
        }
        
        if metrics.cpu_utilization < 0.5 {
            self.increase_concurrency(processor).await?;
        }
        
        if metrics.backpressure_events > 100 {
            self.adjust_flow_control(processor).await?;
        }
        
        Ok(())
    }

    async fn optimize_memory_usage(&self, processor: &mut StreamProcessor) -> Result<(), OptimizationError> {
        // Reduce buffer sizes
        processor.config.buffer_size = (processor.config.buffer_size * 3) / 4;
        
        // Enable more aggressive garbage collection
        processor.enable_aggressive_gc();
        
        Ok(())
    }
}
```

Next: **[Development Tools Integration](development-tools.md)** - IDE and debugging support

---

**Streaming Excellence:** Process unlimited data efficiently with memory-conscious streaming patterns.
