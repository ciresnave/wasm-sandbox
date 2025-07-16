# Streaming APIs for Large Data

The streaming APIs in `wasm-sandbox` allow processing large datasets that don't fit entirely in memory, enabling efficient handling of big files, real-time data, and long-running processes.

## Key Benefits

- **Memory Efficiency**: Process data larger than available RAM
- **Real-Time Processing**: Handle data as it arrives
- **Backpressure Management**: Control flow to prevent memory issues
- **Bidirectional Communication**: Stream data both to and from sandboxed code
- **Multiple Channel Types**: In-memory, file-based, and network streaming

## Core Concepts

### Streaming Channels

Streaming channels are bidirectional pipes that allow continuous data flow between the host and sandboxed WebAssembly code.

```rust
use wasm_sandbox::communication::streaming::{
    StreamingManager, StreamingFactory, StreamingChannel, StreamChunk
};

// Create a streaming manager
let streaming_manager = StreamingManager::new();

// Create an in-memory streaming channel
let stream_channel = streaming_manager.create_channel(
    Some("my-stream".to_string()),
    None, // default configuration
).await?;

println!("Channel ID: {}", stream_channel.id());
```

### Stream Chunks

Data flows through channels as discrete chunks, which can be processed independently:

```rust
// Create a chunk
let chunk = StreamChunk {
    data: "Hello, world!".as_bytes().to_vec(),
    is_final: false,  // Not the final chunk in the stream
    sequence: 0,      // Sequence number
    metadata: None,   // Optional metadata
};

// Send the chunk to the WebAssembly module
stream_channel.send_chunk(chunk).await?;

// Receive a chunk from the WebAssembly module
let received_chunk = stream_channel.receive_chunk().await?;
println!("Received {} bytes", received_chunk.data.len());
```

## Using Streams

### Memory-Based Streaming

Ideal for medium-sized data or when speed is a priority:

```rust
use futures::StreamExt;

// Create a memory streaming channel
let channel = StreamingFactory::create_memory_channel("processor", None);

// Send data chunks
for i in 0..100 {
    let data = format!("Chunk {}", i);
    channel.send_bytes(data.as_bytes(), i == 99).await?;
}

// Receive data as a stream
let mut receiver = channel.receive_stream();
while let Some(chunk_result) = receiver.next().await {
    let chunk = chunk_result?;
    println!("Received chunk {}: {} bytes", chunk.sequence, chunk.data.len());
    
    if chunk.is_final {
        break;
    }
}
```

### File-Based Streaming

Perfect for processing very large files:

```rust
// Create a file streaming channel
let file_stream = StreamingFactory::create_file_channel(
    "file-processor",
    None, // default config
    Some("input.dat"),  // Input file path
    Some("output.dat"), // Output file path
);

// Process the file in chunks
let mut stream = file_stream.receive_stream();
let mut total_processed = 0;

while let Some(chunk_result) = stream.next().await {
    let chunk = chunk_result?;
    
    // Process the chunk
    let processed_data = process_chunk(&chunk.data)?;
    
    // Write processed data to output
    file_stream.send_bytes(&processed_data, chunk.is_final).await?;
    
    total_processed += chunk.data.len();
    println!("Processed: {} bytes", total_processed);
    
    if chunk.is_final {
        break;
    }
}
```

### Stream Transformers

Convert streams between different formats:

```rust
use wasm_sandbox::communication::streaming::StreamTransformer;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Record {
    id: u32,
    name: String,
}

// Transform byte stream to JSON objects
let json_stream = StreamTransformer::bytes_to_json_stream::<_, Record>(byte_stream);

// Process the JSON objects
while let Some(record_result) = json_stream.next().await {
    let record = record_result?;
    println!("Record: id={}, name={}", record.id, record.name);
}
```

## Integration with WebAssembly Sandboxes

### Registering Stream Handlers

```rust
use wasm_sandbox::{WasmSandbox, StreamingInstanceExt};

// Create a sandbox
let sandbox = WasmSandbox::new()?;
let module_id = sandbox.load_module(&wasm_bytes)?;
let instance_id = sandbox.create_instance(module_id, None)?;

// Create a streaming channel
let stream_channel = streaming_manager.create_channel(None, None).await?;

// Call a streaming function in the sandbox
let result: String = sandbox.call_streaming_function(
    instance_id,
    "process_stream",
    &{ "format": "json", "operation": "transform" },
    stream_channel.clone()
).await?;

println!("Stream processing result: {}", result);
```

## Advanced Features

### Backpressure Management

Control data flow to prevent memory issues:

```rust
// Configure a channel with backpressure control
let config = StreamingChannelConfig {
    buffer_size: 16 * 1024, // 16KB buffer
    max_chunk_size: 4 * 1024, // 4KB max chunk size
    ..Default::default()
};

let channel = StreamingFactory::create_memory_channel("controlled", Some(config));
```

### Stream Statistics

Monitor stream performance:

```rust
// Get statistics for a channel
let stats = stream_channel.stats();
println!("Bytes sent: {}", stats.bytes_sent);
println!("Bytes received: {}", stats.bytes_received);
println!("Chunks processed: {}", stats.chunks_sent + stats.chunks_received);
println!("Average chunk size: {:.2} bytes", stats.average_chunk_size);
```

### Multiple Parallel Streams

Process multiple streams in parallel:

```rust
use futures::future::join_all;

// Create multiple streams
let mut streams = Vec::new();
for i in 0..10 {
    let channel = streaming_manager.create_channel(
        Some(format!("stream-{}", i)),
        None
    ).await?;
    streams.push(channel);
}

// Process all streams in parallel
let tasks: Vec<_> = streams.iter().map(|stream| {
    process_stream(sandbox.clone(), stream.clone())
}).collect();

let results = join_all(tasks).await;
```

## Use Cases

- **Large File Processing**: Process gigabytes or terabytes of data
- **Media Processing**: Stream audio/video through transformations
- **Real-time Analytics**: Process continuous data streams
- **ETL Pipelines**: Extract, transform, and load data at scale
- **IoT Data**: Handle continuous sensor data streams

## Performance Optimization

- Use appropriate buffer sizes for your use case
- Consider file-based streaming for very large datasets
- Use small chunks for real-time processing, larger chunks for batch processing
- Monitor stream statistics to identify bottlenecks
- Use async streams for non-blocking operation
