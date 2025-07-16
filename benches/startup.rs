//! Benchmarks for the Wasm sandbox
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use wasm_sandbox::WasmSandbox;
use tokio::runtime::Runtime;

// WASM module for benchmarking - simple add function
const TEST_MODULE: &[u8] = include_bytes!("../fixtures/test_module.wasm");

fn bench_module_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("module_loading");
    
    group.bench_function("load_module", |b| {
        b.iter(|| {
            let sandbox = WasmSandbox::new().unwrap();
            black_box(sandbox.load_module(TEST_MODULE).unwrap());
        });
    });
    
    group.finish();
}

fn bench_instance_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_creation");
    
    group.bench_function("create_instance", |b| {
        b.iter(|| {
            // Setup
            let mut sandbox = WasmSandbox::new().unwrap();
            let module_id = sandbox.load_module(TEST_MODULE).unwrap();
            let instance_id = sandbox.create_instance(module_id, None).unwrap();
            black_box(sandbox.remove_instance(instance_id));
        });
    });
    
    group.finish();
}

fn bench_function_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("function_calls");
    
    // Benchmark function calls with different sizes
    for size in [2, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("add", size), size, |b, &_size| {
            b.iter(|| {
                let rt = Runtime::new().unwrap();
                let mut sandbox = WasmSandbox::new().unwrap();
                let module_id = sandbox.load_module(TEST_MODULE).unwrap();
                let instance_id = sandbox.create_instance(module_id, None).unwrap();
                
                let result = rt.block_on(async {
                    sandbox.call_function::<_, i32>(instance_id, "add", &(5, 7)).await
                });
                black_box(result.unwrap());
            });
        });
    }
    
    group.finish();
}

fn bench_startup_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup_time");
    
    let rt = Runtime::new().unwrap();
    
    group.bench_function("full_startup", |b| {
        b.iter(|| {
            let result = rt.block_on(async {
                let mut sandbox = WasmSandbox::new().unwrap();
                let module_id = sandbox.load_module(TEST_MODULE).unwrap();
                let instance_id = sandbox.create_instance(module_id, None).unwrap();
                
                // Perform one function call to ensure everything is loaded
                sandbox.call_function::<_, i32>(instance_id, "add", &(5, 7)).await
            });
            black_box(result.unwrap());
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_module_loading, bench_instance_creation, bench_function_calls, bench_startup_time);
criterion_main!(benches);
