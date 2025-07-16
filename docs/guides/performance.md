# Performance Guide

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)** | 🚀 **[API Reference](https://docs.rs/wasm-sandbox)**

Comprehensive performance optimization strategies, benchmarking, and tuning for high-performance wasm-sandbox applications.

## Performance Overview

Key performance areas:

- **Compilation Speed** - Fast module compilation and caching
- **Execution Speed** - Optimized runtime performance  
- **Memory Efficiency** - Minimal memory overhead
- **Startup Time** - Fast sandbox initialization
- **Throughput** - High concurrent execution capacity

## Quick Performance Setup

```rust
use wasm_sandbox::{WasmSandbox, PerformanceConfig, OptimizationLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = WasmSandbox::builder()
        .performance_config(PerformanceConfig {
            optimization_level: OptimizationLevel::Aggressive,
            enable_jit: true,
            enable_caching: true,
            parallel_compilation: true,
            memory_pool_size: 64 * 1024 * 1024, // 64MB
            ..Default::default()
        })
        .enable_metrics(true)
        .build()
        .await?;

    // Pre-compile and cache modules
    let module = sandbox.compile_with_cache("fast_module.wasm").await?;
    
    // Warm up JIT
    sandbox.warmup(&module).await?;

    // High-performance execution
    let start = std::time::Instant::now();
    let result: Vec<i32> = sandbox.call("process_batch", &input_data).await?;
    let duration = start.elapsed();
    
    println!("Processed {} items in {:?}", result.len(), duration);
    
    Ok(())
}
```

## Compilation Optimization

### Compilation Caching

```rust
pub struct CompilationCache {
    cache_dir: PathBuf,
    cache_entries: HashMap<ModuleHash, CacheEntry>,
    max_cache_size: usize,
    eviction_policy: EvictionPolicy,
}

impl CompilationCache {
    pub async fn get_or_compile(&mut self, wasm_bytes: &[u8], config: &CompilationConfig) -> Result<CompiledModule, CacheError> {
        let hash = self.compute_hash(wasm_bytes, config);
        
        // Check cache first
        if let Some(entry) = self.cache_entries.get(&hash) {
            if entry.is_valid() {
                return self.load_from_cache(&entry.path).await;
            }
        }

        // Compile and cache
        let module = self.compile_module(wasm_bytes, config).await?;
        self.store_in_cache(hash, &module).await?;
        
        Ok(module)
    }

    async fn compile_module(&self, wasm_bytes: &[u8], config: &CompilationConfig) -> Result<CompiledModule, CompilationError> {
        let start = Instant::now();
        
        // Use parallel compilation for large modules
        let module = if wasm_bytes.len() > 1024 * 1024 && config.parallel_compilation {
            self.compile_parallel(wasm_bytes, config).await?
        } else {
            self.compile_sequential(wasm_bytes, config).await?
        };

        let compilation_time = start.elapsed();
        self.metrics.record_compilation_time(compilation_time);
        
        Ok(module)
    }

    async fn compile_parallel(&self, wasm_bytes: &[u8], config: &CompilationConfig) -> Result<CompiledModule, CompilationError> {
        // Split large modules into chunks for parallel compilation
        let chunk_size = wasm_bytes.len() / num_cpus::get();
        let chunks: Vec<_> = wasm_bytes.chunks(chunk_size).collect();
        
        let futures: Vec<_> = chunks.into_iter().enumerate().map(|(i, chunk)| {
            let config = config.clone();
            async move {
                self.compile_chunk(i, chunk, &config).await
            }
        }).collect();

        let compiled_chunks = futures::future::try_join_all(futures).await?;
        self.link_chunks(compiled_chunks).await
    }
}
```

### JIT Optimization

```rust
pub struct JitOptimizer {
    hot_functions: HashMap<String, HotFunction>,
    call_counts: HashMap<String, u64>,
    optimization_threshold: u64,
    tier_up_delay: Duration,
}

#[derive(Debug)]
struct HotFunction {
    name: String,
    call_count: u64,
    average_execution_time: Duration,
    optimization_level: OptimizationTier,
    last_optimized: Instant,
}

#[derive(Debug, Clone, PartialEq)]
enum OptimizationTier {
    Interpreter,
    Baseline,
    Optimized,
    HighlyOptimized,
}

impl JitOptimizer {
    pub async fn maybe_optimize(&mut self, function_name: &str, execution_time: Duration) -> bool {
        let call_count = self.call_counts.entry(function_name.to_string()).or_insert(0);
        *call_count += 1;

        let hot_function = self.hot_functions.entry(function_name.to_string()).or_insert(HotFunction {
            name: function_name.to_string(),
            call_count: 0,
            average_execution_time: Duration::ZERO,
            optimization_level: OptimizationTier::Interpreter,
            last_optimized: Instant::now(),
        });

        // Update execution time average
        hot_function.average_execution_time = 
            (hot_function.average_execution_time * hot_function.call_count as u32 + execution_time) / (hot_function.call_count + 1) as u32;
        hot_function.call_count = *call_count;

        // Check if function should be tier-upped
        if self.should_tier_up(hot_function) {
            self.tier_up_function(hot_function).await
        } else {
            false
        }
    }

    fn should_tier_up(&self, function: &HotFunction) -> bool {
        let since_last_opt = function.last_optimized.elapsed();
        
        match function.optimization_level {
            OptimizationTier::Interpreter => {
                function.call_count >= 100 && since_last_opt > Duration::from_millis(10)
            }
            OptimizationTier::Baseline => {
                function.call_count >= 1000 && since_last_opt > Duration::from_millis(100)
            }
            OptimizationTier::Optimized => {
                function.call_count >= 10000 && since_last_opt > Duration::from_secs(1)
            }
            OptimizationTier::HighlyOptimized => false,
        }
    }

    async fn tier_up_function(&mut self, function: &mut HotFunction) -> bool {
        let new_tier = match function.optimization_level {
            OptimizationTier::Interpreter => OptimizationTier::Baseline,
            OptimizationTier::Baseline => OptimizationTier::Optimized,
            OptimizationTier::Optimized => OptimizationTier::HighlyOptimized,
            OptimizationTier::HighlyOptimized => return false,
        };

        // Trigger recompilation with higher optimization
        if self.recompile_with_optimization(&function.name, new_tier).await.is_ok() {
            function.optimization_level = new_tier;
            function.last_optimized = Instant::now();
            true
        } else {
            false
        }
    }
}
```

## Memory Optimization

### Memory Pooling

```rust
pub struct MemoryPool {
    pools: Vec<Pool>,
    allocation_stats: AllocationStats,
}

struct Pool {
    size_class: usize,
    available: Vec<MemoryBlock>,
    allocated: HashSet<*mut u8>,
    total_capacity: usize,
}

impl MemoryPool {
    pub fn new() -> Self {
        let mut pools = Vec::new();
        
        // Create pools for common allocation sizes
        let size_classes = vec![
            64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536
        ];
        
        for size in size_classes {
            pools.push(Pool {
                size_class: size,
                available: Vec::with_capacity(100),
                allocated: HashSet::new(),
                total_capacity: 100 * size,
            });
        }
        
        Self {
            pools,
            allocation_stats: AllocationStats::new(),
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<*mut u8> {
        let pool_index = self.find_pool_for_size(size)?;
        let pool = &mut self.pools[pool_index];
        
        if let Some(block) = pool.available.pop() {
            pool.allocated.insert(block.ptr);
            self.allocation_stats.record_allocation(size);
            Some(block.ptr)
        } else if pool.allocated.len() * pool.size_class < pool.total_capacity {
            // Allocate new block
            let layout = std::alloc::Layout::from_size_align(pool.size_class, 8).ok()?;
            let ptr = unsafe { std::alloc::alloc(layout) };
            
            if !ptr.is_null() {
                pool.allocated.insert(ptr);
                self.allocation_stats.record_allocation(size);
                Some(ptr)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn deallocate(&mut self, ptr: *mut u8, size: usize) {
        if let Some(pool_index) = self.find_pool_for_size(size) {
            let pool = &mut self.pools[pool_index];
            
            if pool.allocated.remove(&ptr) {
                pool.available.push(MemoryBlock { ptr });
                self.allocation_stats.record_deallocation(size);
            }
        }
    }
}
```

### Memory-Mapped I/O

```rust
pub struct MappedMemoryManager {
    mappings: HashMap<String, MappedRegion>,
    page_size: usize,
}

struct MappedRegion {
    file_path: PathBuf,
    mapping: memmap2::Mmap,
    size: usize,
    access_count: AtomicU64,
    last_accessed: AtomicU64,
}

impl MappedMemoryManager {
    pub async fn map_file(&mut self, file_path: &Path, size: Option<usize>) -> Result<&[u8], MappingError> {
        let key = file_path.to_string_lossy().to_string();
        
        if let Some(region) = self.mappings.get(&key) {
            region.access_count.fetch_add(1, Ordering::Relaxed);
            region.last_accessed.store(
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                Ordering::Relaxed
            );
            return Ok(&region.mapping);
        }

        // Create new mapping
        let file = std::fs::File::open(file_path)?;
        let mapping = unsafe { memmap2::Mmap::map(&file)? };
        
        let region = MappedRegion {
            file_path: file_path.to_path_buf(),
            size: mapping.len(),
            mapping,
            access_count: AtomicU64::new(1),
            last_accessed: AtomicU64::new(
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
            ),
        };

        let mapping_ref = &region.mapping;
        self.mappings.insert(key, region);
        
        Ok(mapping_ref)
    }

    pub async fn unmap_unused(&mut self, max_age: Duration) {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let max_age_secs = max_age.as_secs();
        
        self.mappings.retain(|_, region| {
            let last_accessed = region.last_accessed.load(Ordering::Relaxed);
            now - last_accessed < max_age_secs
        });
    }
}
```

## Execution Optimization

### Function Call Optimization

```rust
pub struct CallOptimizer {
    call_cache: LruCache<CallSignature, CachedResult>,
    inline_threshold: usize,
    batch_threshold: usize,
}

#[derive(Hash, PartialEq, Eq)]
struct CallSignature {
    function_name: String,
    args_hash: u64,
}

impl CallOptimizer {
    pub async fn optimize_call<T, R>(&mut self, sandbox: &WasmSandbox, function: &str, args: &T) -> Result<R, CallError>
    where
        T: serde::Serialize + std::hash::Hash,
        R: for<'de> serde::Deserialize<'de> + Clone,
    {
        let signature = CallSignature {
            function_name: function.to_string(),
            args_hash: self.hash_args(args),
        };

        // Check cache for pure functions
        if self.is_pure_function(function) {
            if let Some(cached) = self.call_cache.get(&signature) {
                if !cached.is_expired() {
                    return Ok(cached.result.clone());
                }
            }
        }

        // Check if function should be inlined
        if self.should_inline(function) {
            return self.inline_call(sandbox, function, args).await;
        }

        // Regular function call
        let result = sandbox.call(function, args).await?;

        // Cache result for pure functions
        if self.is_pure_function(function) {
            self.call_cache.put(signature, CachedResult {
                result: result.clone(),
                timestamp: Instant::now(),
                ttl: Duration::from_secs(300), // 5 minutes
            });
        }

        Ok(result)
    }

    async fn inline_call<T, R>(&self, sandbox: &WasmSandbox, function: &str, args: &T) -> Result<R, CallError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de>,
    {
        // Get function bytecode
        let bytecode = sandbox.get_function_bytecode(function).await?;
        
        // Inline execution (pseudo-code)
        let result = sandbox.execute_bytecode_inline(bytecode, args).await?;
        
        Ok(result)
    }
}
```

### Batch Processing

```rust
pub struct BatchProcessor {
    batch_size: usize,
    batch_timeout: Duration,
    pending_calls: Vec<PendingCall>,
    result_senders: HashMap<CallId, tokio::sync::oneshot::Sender<CallResult>>,
}

struct PendingCall {
    id: CallId,
    function: String,
    args: serde_json::Value,
    timestamp: Instant,
}

impl BatchProcessor {
    pub async fn call_batched<T, R>(&mut self, function: &str, args: T) -> Result<R, BatchError>
    where
        T: serde::Serialize,
        R: for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let call_id = CallId::new();
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // Add to batch
        self.pending_calls.push(PendingCall {
            id: call_id,
            function: function.to_string(),
            args: serde_json::to_value(args)?,
            timestamp: Instant::now(),
        });
        
        self.result_senders.insert(call_id, tx);

        // Check if we should flush the batch
        if self.should_flush_batch() {
            self.flush_batch().await?;
        }

        // Wait for result
        let result = rx.await??;
        let typed_result = serde_json::from_value(result)?;
        
        Ok(typed_result)
    }

    async fn flush_batch(&mut self) -> Result<(), BatchError> {
        if self.pending_calls.is_empty() {
            return Ok(());
        }

        // Group calls by function
        let mut function_batches: HashMap<String, Vec<PendingCall>> = HashMap::new();
        
        for call in self.pending_calls.drain(..) {
            function_batches.entry(call.function.clone()).or_default().push(call);
        }

        // Execute batches in parallel
        let futures: Vec<_> = function_batches.into_iter().map(|(function, calls)| {
            self.execute_function_batch(function, calls)
        }).collect();

        futures::future::try_join_all(futures).await?;
        
        Ok(())
    }

    async fn execute_function_batch(&self, function: String, calls: Vec<PendingCall>) -> Result<(), BatchError> {
        // Prepare batch arguments
        let batch_args: Vec<_> = calls.iter().map(|call| call.args.clone()).collect();
        
        // Execute batch
        let batch_results: Vec<serde_json::Value> = self.sandbox.call_batch(&function, &batch_args).await?;
        
        // Send results back
        for (call, result) in calls.into_iter().zip(batch_results.into_iter()) {
            if let Some(sender) = self.result_senders.remove(&call.id) {
                let _ = sender.send(Ok(result));
            }
        }
        
        Ok(())
    }
}
```

## Benchmarking

```rust
pub struct PerformanceBenchmark {
    test_cases: Vec<BenchmarkCase>,
    baseline_results: Option<BenchmarkResults>,
}

struct BenchmarkCase {
    name: String,
    function: String,
    input_size: usize,
    iterations: usize,
    warmup_iterations: usize,
}

impl PerformanceBenchmark {
    pub async fn run_comprehensive_benchmark(&mut self, sandbox: &WasmSandbox) -> BenchmarkReport {
        let mut results = BenchmarkReport::new();
        
        for case in &self.test_cases {
            let case_result = self.run_benchmark_case(sandbox, case).await;
            results.add_case_result(case.name.clone(), case_result);
        }
        
        // Compare with baseline if available
        if let Some(baseline) = &self.baseline_results {
            results.performance_delta = Some(results.compare_with_baseline(baseline));
        }
        
        results
    }

    async fn run_benchmark_case(&self, sandbox: &WasmSandbox, case: &BenchmarkCase) -> CaseResult {
        let input_data = self.generate_test_data(case.input_size);
        
        // Warmup
        for _ in 0..case.warmup_iterations {
            let _ = sandbox.call(&case.function, &input_data).await;
        }

        // Actual benchmark
        let mut execution_times = Vec::with_capacity(case.iterations);
        let mut memory_usage = Vec::with_capacity(case.iterations);
        
        for _ in 0..case.iterations {
            let start_memory = sandbox.get_memory_usage().await.unwrap_or(0);
            let start_time = Instant::now();
            
            let _result = sandbox.call(&case.function, &input_data).await;
            
            let end_time = Instant::now();
            let end_memory = sandbox.get_memory_usage().await.unwrap_or(0);
            
            execution_times.push(end_time.duration_since(start_time));
            memory_usage.push(end_memory.saturating_sub(start_memory));
        }

        CaseResult {
            execution_times,
            memory_usage,
            throughput: self.calculate_throughput(&execution_times, case.input_size),
            statistics: self.calculate_statistics(&execution_times),
        }
    }
}
```

Next: **[Benchmarks](benchmarks.md)** - Performance benchmarking suite

---

**Performance Excellence:** Achieve maximum performance through systematic optimization, caching, and intelligent resource management.
