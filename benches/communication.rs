//! Benchmarks for the different communication channels
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use wasm_sandbox::{
    WasmSandbox,
    communication::{
        MemoryChannel, MemoryChannelConfig, MemoryRpcChannel,
        memory::SharedMemoryRegion, CommunicationChannel, RpcChannel,
    },
};
use std::sync::Arc;
use tokio::runtime::Runtime;

// WASM module for benchmarking communication
const TEST_MODULE: &[u8] = include_bytes!("../fixtures/test_module.wasm");

fn bench_memory_channel_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_channel_throughput");
    
    // Create memory regions for testing
    let config = MemoryChannelConfig::default();
    let data_region = Arc::new(SharedMemoryRegion::new("data", config.data_size));
    let control_region = Arc::new(SharedMemoryRegion::new("control", config.control_size));
    
    // Create memory channel
    let memory_channel = MemoryChannel::new(&config, data_region, control_region);
    
    // Benchmark different message sizes
    for size in [10, 100, 1000, 10000].iter() {
        let message = vec![0u8; *size];
        
        group.bench_with_input(BenchmarkId::new("send_receive", size), size, |b, &_size| {
            b.iter(|| {
                // For benchmarking, we'll just test the basic send_to_guest method
                // Note: This is a simplified benchmark since proper guest communication
                // requires a full WASM instance setup
                let result = memory_channel.send_to_guest(&message);
                let _ = black_box(result);
            });
        });
    }
    
    group.finish();
}

fn bench_rpc_channel_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("rpc_channel_calls");
    
    // Create memory regions and channel
    let config = MemoryChannelConfig::default();
    let data_region = Arc::new(SharedMemoryRegion::new("rpc_data", config.data_size));
    let control_region = Arc::new(SharedMemoryRegion::new("rpc_control", config.control_size));
    let memory_channel = Arc::new(MemoryChannel::new(&config, data_region, control_region));
    
    let mut rpc_channel = MemoryRpcChannel::new(memory_channel);
    
    // Register a test function
    let test_func = Box::new(|data: &str| -> wasm_sandbox::Result<String> {
        Ok(format!("Echo: {data}"))
    });
    rpc_channel.register_host_function_json("test_echo", test_func).unwrap();
    
    // Benchmark different message sizes
    for size in [10, 100, 1000].iter() {
        let test_data = "x".repeat(*size);
        
        group.bench_with_input(BenchmarkId::new("json_call", size), size, |b, &_size| {
            b.iter(|| {
                // Note: This will likely fail since we don't have a guest to respond,
                // but it tests the call preparation and sending
                let result = rpc_channel.call_guest_function_json("test_echo", &test_data);
                let _ = black_box(result);
            });
        });
    }
    
    group.finish();
}

fn bench_sandbox_with_communication(c: &mut Criterion) {
    let mut group = c.benchmark_group("sandbox_communication");
    
    let rt = Runtime::new().unwrap();
    
    group.bench_function("sandbox_setup_with_module", |b| {
        b.iter(|| {
            let result = rt.block_on(async {
                let mut sandbox = WasmSandbox::new().unwrap();
                let module_id = sandbox.load_module(TEST_MODULE).unwrap();
                let instance_id = sandbox.create_instance(module_id, None).unwrap();
                
                // Basic function call to test the setup
                sandbox.call_function::<_, i32>(instance_id, "add", &(5, 7)).await
            });
            let _ = black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_channel_throughput,
    bench_rpc_channel_calls,
    bench_sandbox_with_communication
);
criterion_main!(benches);
