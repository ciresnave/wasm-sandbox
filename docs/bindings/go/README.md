# Go Bindings

📖 **[← Back to Documentation](../../README.md)** | 🏠 **[← Main README](../../../README.md)** | 🐹 **[Go Package](https://pkg.go.dev/github.com/your-org/wasm-sandbox-go)**

Native Go bindings for wasm-sandbox, enabling secure WebAssembly execution in Go applications with full goroutine support and idiomatic Go APIs.

## Installation

```bash
go get github.com/your-org/wasm-sandbox-go
```

## Quick Start

### Basic Usage

```go
package main

import (
    "context"
    "fmt"
    "log"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    "github.com/your-org/wasm-sandbox-go/pkg/security"
)

func main() {
    ctx := context.Background()
    
    // Create a secure sandbox
    sb, err := sandbox.NewBuilder().
        Source("./calculator.wasm").
        SecurityPolicy(security.StrictPolicy()).
        MemoryLimit(64 * 1024 * 1024). // 64MB
        CPUTimeout(30 * time.Second).
        Build(ctx)
    if err != nil {
        log.Fatal(err)
    }
    defer sb.Close()

    // Call a function
    result, err := sb.Call(ctx, "add", 5, 3)
    if err != nil {
        log.Fatal(err)
    }
    
    fmt.Printf("Result: %v\n", result) // 8
}
```

### Error Handling

```go
package main

import (
    "context"
    "errors"
    "fmt"
    "log"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    "github.com/your-org/wasm-sandbox-go/pkg/security"
)

func main() {
    ctx := context.Background()
    
    sb, err := sandbox.FromFile(ctx, "./calculator.wasm")
    if err != nil {
        log.Fatal(err)
    }
    defer sb.Close()

    result, err := sb.Call(ctx, "divide", 10, 0)
    if err != nil {
        var secErr *security.ViolationError
        var resErr *sandbox.ResourceError
        var execErr *sandbox.ExecutionError
        
        switch {
        case errors.As(err, &secErr):
            fmt.Printf("Security violation: %v\n", secErr)
        case errors.As(err, &resErr):
            fmt.Printf("Resource limit exceeded: %v\n", resErr)
        case errors.As(err, &execErr):
            fmt.Printf("Execution error: %v\n", execErr)
        default:
            fmt.Printf("Unknown error: %v\n", err)
        }
        return
    }
    
    fmt.Printf("Result: %v\n", result)
}
```

## API Reference

### Sandbox Interface

```go
// Sandbox represents a WebAssembly sandbox instance
type Sandbox interface {
    // Call executes a function in the sandbox
    Call(ctx context.Context, function string, args ...interface{}) (interface{}, error)
    
    // CallWithTimeout executes a function with a custom timeout
    CallWithTimeout(ctx context.Context, function string, timeout time.Duration, args ...interface{}) (interface{}, error)
    
    // CallBatch executes multiple functions in batch
    CallBatch(ctx context.Context, calls []FunctionCall) ([]interface{}, error)
    
    // CallWithAudit executes a function with audit logging
    CallWithAudit(ctx context.Context, function string, args ...interface{}) (*AuditResult, error)
    
    // MemoryUsage returns current memory usage in bytes
    MemoryUsage(ctx context.Context) (int64, error)
    
    // ResourceStats returns comprehensive resource statistics
    ResourceStats(ctx context.Context) (*ResourceStats, error)
    
    // Reset resets the sandbox state
    Reset(ctx context.Context) error
    
    // Close disposes of the sandbox and frees resources
    Close() error
}

// FunctionCall represents a function call in batch operations
type FunctionCall struct {
    Function string        `json:"function"`
    Args     []interface{} `json:"args"`
}

// AuditResult contains execution results with audit information
type AuditResult struct {
    Output     interface{}           `json:"output"`
    Violations []security.Violation  `json:"violations"`
    Events     []security.Event      `json:"events"`
    Stats      *ResourceStats        `json:"stats"`
}

// ResourceStats contains resource usage statistics
type ResourceStats struct {
    CPUTimeMs        int64 `json:"cpu_time_ms"`
    PeakMemoryBytes  int64 `json:"peak_memory_bytes"`
    FunctionCalls    int64 `json:"function_calls"`
    NetworkRequests  int64 `json:"network_requests"`
    FileSystemOps    int64 `json:"filesystem_ops"`
}
```

### Builder Pattern

```go
// Builder provides a fluent interface for creating sandboxes
type Builder struct {
    // private fields
}

// NewBuilder creates a new sandbox builder
func NewBuilder() *Builder

// Source sets the WebAssembly module source
func (b *Builder) Source(source interface{}) *Builder

// SecurityPolicy sets the security policy
func (b *Builder) SecurityPolicy(policy *security.Policy) *Builder

// MemoryLimit sets the memory limit in bytes
func (b *Builder) MemoryLimit(bytes int64) *Builder

// CPUTimeout sets the CPU execution timeout
func (b *Builder) CPUTimeout(timeout time.Duration) *Builder

// AddCapability adds a security capability
func (b *Builder) AddCapability(capability security.Capability) *Builder

// InstanceID sets a unique instance identifier
func (b *Builder) InstanceID(id string) *Builder

// Build creates the sandbox instance
func (b *Builder) Build(ctx context.Context) (Sandbox, error)

// Convenience constructors
func FromFile(ctx context.Context, path string) (Sandbox, error)
func FromBytes(ctx context.Context, data []byte) (Sandbox, error)
func FromURL(ctx context.Context, url string) (Sandbox, error)
```

### Security Policies

```go
package security

// Policy represents a security policy configuration
type Policy struct {
    // private fields
}

// Predefined policies
func StrictPolicy() *Policy
func ModeratePolicy() *Policy
func PermissivePolicy() *Policy

// Policy builder
func NewPolicyBuilder() *PolicyBuilder

type PolicyBuilder struct {
    // private fields
}

func (pb *PolicyBuilder) AllowNetworkAccess(domains []string) *PolicyBuilder
func (pb *PolicyBuilder) AllowFileSystemAccess(paths []string, readOnly bool) *PolicyBuilder
func (pb *PolicyBuilder) AllowEnvironmentAccess(vars []string) *PolicyBuilder
func (pb *PolicyBuilder) EnableAuditLogging(enabled bool) *PolicyBuilder
func (pb *PolicyBuilder) MaxExecutionTime(timeout time.Duration) *PolicyBuilder
func (pb *PolicyBuilder) Build() *Policy

// Capabilities
type Capability interface {
    Type() string
    Validate() error
}

func NetworkAccess(options NetworkAccessOptions) Capability
func FileSystemAccess(options FileSystemAccessOptions) Capability
func EnvironmentAccess(options EnvironmentAccessOptions) Capability

type NetworkAccessOptions struct {
    Domains         []string
    AllowedPorts    []int
    RequireHTTPS    bool
    MaxRequestSize  int64
}

type FileSystemAccessOptions struct {
    Paths       []string
    ReadOnly    bool
    MaxFileSize int64
}

type EnvironmentAccessOptions struct {
    Variables []string
}
```

## Advanced Features

### HTTP Server Integration

```go
package main

import (
    "context"
    "encoding/json"
    "fmt"
    "log"
    "net/http"
    "sync"
    "time"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    "github.com/your-org/wasm-sandbox-go/pkg/security"
)

// SandboxPool manages a pool of sandbox instances
type SandboxPool struct {
    sandboxes chan sandbox.Sandbox
    mu        sync.RWMutex
    closed    bool
}

// NewSandboxPool creates a new sandbox pool
func NewSandboxPool(ctx context.Context, size int, source string) (*SandboxPool, error) {
    pool := &SandboxPool{
        sandboxes: make(chan sandbox.Sandbox, size),
    }
    
    // Initialize pool with sandboxes
    for i := 0; i < size; i++ {
        sb, err := sandbox.NewBuilder().
            Source(source).
            SecurityPolicy(security.ModeratePolicy()).
            InstanceID(fmt.Sprintf("pool-%d", i)).
            Build(ctx)
        if err != nil {
            pool.Close()
            return nil, err
        }
        
        pool.sandboxes <- sb
    }
    
    return pool, nil
}

// Acquire gets a sandbox from the pool
func (p *SandboxPool) Acquire(ctx context.Context) (sandbox.Sandbox, error) {
    p.mu.RLock()
    defer p.mu.RUnlock()
    
    if p.closed {
        return nil, fmt.Errorf("pool is closed")
    }
    
    select {
    case sb := <-p.sandboxes:
        return sb, nil
    case <-ctx.Done():
        return nil, ctx.Err()
    }
}

// Release returns a sandbox to the pool
func (p *SandboxPool) Release(sb sandbox.Sandbox) {
    p.mu.RLock()
    defer p.mu.RUnlock()
    
    if p.closed {
        sb.Close()
        return
    }
    
    select {
    case p.sandboxes <- sb:
        // Successfully returned to pool
    default:
        // Pool is full, dispose of sandbox
        sb.Close()
    }
}

// Close closes the pool and all sandboxes
func (p *SandboxPool) Close() {
    p.mu.Lock()
    defer p.mu.Unlock()
    
    if p.closed {
        return
    }
    
    p.closed = true
    close(p.sandboxes)
    
    // Close all sandboxes
    for sb := range p.sandboxes {
        sb.Close()
    }
}

// ProcessRequest represents an incoming request
type ProcessRequest struct {
    Data     map[string]interface{} `json:"data"`
    Function string                 `json:"function"`
}

// ProcessResponse represents the response
type ProcessResponse struct {
    Result  interface{} `json:"result,omitempty"`
    Error   string      `json:"error,omitempty"`
    Success bool        `json:"success"`
}

// HTTP handler using the sandbox pool
func main() {
    ctx := context.Background()
    
    // Create sandbox pool
    pool, err := NewSandboxPool(ctx, 10, "./request-handler.wasm")
    if err != nil {
        log.Fatal(err)
    }
    defer pool.Close()
    
    // HTTP handler
    http.HandleFunc("/process", func(w http.ResponseWriter, r *http.Request) {
        if r.Method != http.MethodPost {
            http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
            return
        }
        
        var req ProcessRequest
        if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
            http.Error(w, "Invalid JSON", http.StatusBadRequest)
            return
        }
        
        // Acquire sandbox from pool
        ctx, cancel := context.WithTimeout(r.Context(), 30*time.Second)
        defer cancel()
        
        sb, err := pool.Acquire(ctx)
        if err != nil {
            http.Error(w, "No available sandboxes", http.StatusServiceUnavailable)
            return
        }
        defer pool.Release(sb)
        
        // Process request
        result, err := sb.Call(ctx, req.Function, req.Data)
        
        var resp ProcessResponse
        if err != nil {
            resp = ProcessResponse{
                Error:   err.Error(),
                Success: false,
            }
        } else {
            resp = ProcessResponse{
                Result:  result,
                Success: true,
            }
        }
        
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(resp)
    })
    
    fmt.Println("Server starting on :8080")
    log.Fatal(http.ListenAndServe(":8080", nil))
}
```

### Goroutine Integration

```go
package main

import (
    "context"
    "fmt"
    "log"
    "sync"
    "time"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
)

// Worker represents a worker goroutine with its own sandbox
type Worker struct {
    id      int
    sandbox sandbox.Sandbox
    jobs    chan Job
    wg      *sync.WaitGroup
}

// Job represents work to be processed
type Job struct {
    ID       int
    Function string
    Args     []interface{}
    Result   chan JobResult
}

// JobResult represents the result of a job
type JobResult struct {
    Value interface{}
    Error error
}

// NewWorker creates a new worker
func NewWorker(ctx context.Context, id int, source string, wg *sync.WaitGroup) (*Worker, error) {
    sb, err := sandbox.FromFile(ctx, source)
    if err != nil {
        return nil, err
    }
    
    worker := &Worker{
        id:      id,
        sandbox: sb,
        jobs:    make(chan Job, 10),
        wg:      wg,
    }
    
    // Start worker goroutine
    go worker.run(ctx)
    
    return worker, nil
}

// run processes jobs in the worker goroutine
func (w *Worker) run(ctx context.Context) {
    defer w.wg.Done()
    defer w.sandbox.Close()
    
    fmt.Printf("Worker %d started\n", w.id)
    
    for {
        select {
        case job := <-w.jobs:
            result, err := w.sandbox.Call(ctx, job.Function, job.Args...)
            job.Result <- JobResult{Value: result, Error: err}
            
        case <-ctx.Done():
            fmt.Printf("Worker %d stopping\n", w.id)
            return
        }
    }
}

// Submit submits a job to the worker
func (w *Worker) Submit(job Job) {
    w.jobs <- job
}

// WorkerPool manages multiple workers
type WorkerPool struct {
    workers []*Worker
    next    int
    mu      sync.Mutex
}

// NewWorkerPool creates a new worker pool
func NewWorkerPool(ctx context.Context, numWorkers int, source string) (*WorkerPool, error) {
    var wg sync.WaitGroup
    workers := make([]*Worker, numWorkers)
    
    for i := 0; i < numWorkers; i++ {
        wg.Add(1)
        worker, err := NewWorker(ctx, i, source, &wg)
        if err != nil {
            return nil, err
        }
        workers[i] = worker
    }
    
    return &WorkerPool{
        workers: workers,
    }, nil
}

// Execute submits a job to the next available worker
func (wp *WorkerPool) Execute(ctx context.Context, function string, args ...interface{}) (interface{}, error) {
    wp.mu.Lock()
    worker := wp.workers[wp.next]
    wp.next = (wp.next + 1) % len(wp.workers)
    wp.mu.Unlock()
    
    resultChan := make(chan JobResult, 1)
    job := Job{
        Function: function,
        Args:     args,
        Result:   resultChan,
    }
    
    worker.Submit(job)
    
    select {
    case result := <-resultChan:
        return result.Value, result.Error
    case <-ctx.Done():
        return nil, ctx.Err()
    }
}

// Example usage
func main() {
    ctx, cancel := context.WithCancel(context.Background())
    defer cancel()
    
    // Create worker pool
    pool, err := NewWorkerPool(ctx, 5, "./calculator.wasm")
    if err != nil {
        log.Fatal(err)
    }
    
    // Submit multiple jobs concurrently
    var wg sync.WaitGroup
    for i := 0; i < 100; i++ {
        wg.Add(1)
        go func(i int) {
            defer wg.Done()
            
            result, err := pool.Execute(ctx, "multiply", i, 2)
            if err != nil {
                fmt.Printf("Job %d failed: %v\n", i, err)
                return
            }
            
            fmt.Printf("Job %d result: %v\n", i, result)
        }(i)
    }
    
    wg.Wait()
    fmt.Println("All jobs completed")
}
```

### gRPC Integration

```go
package main

import (
    "context"
    "log"
    "net"
    
    "google.golang.org/grpc"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    pb "github.com/your-org/wasm-sandbox-go/proto" // Generated protobuf code
)

// SandboxServiceServer implements the gRPC service
type SandboxServiceServer struct {
    pb.UnimplementedSandboxServiceServer
    pool *SandboxPool
}

// Execute implements the Execute RPC method
func (s *SandboxServiceServer) Execute(ctx context.Context, req *pb.ExecuteRequest) (*pb.ExecuteResponse, error) {
    // Acquire sandbox from pool
    sb, err := s.pool.Acquire(ctx)
    if err != nil {
        return nil, status.Errorf(codes.Unavailable, "no available sandboxes: %v", err)
    }
    defer s.pool.Release(sb)
    
    // Convert protobuf args to Go values
    args := make([]interface{}, len(req.Args))
    for i, arg := range req.Args {
        args[i] = protoValueToInterface(arg)
    }
    
    // Execute function
    result, err := sb.Call(ctx, req.Function, args...)
    if err != nil {
        return nil, status.Errorf(codes.Internal, "execution failed: %v", err)
    }
    
    // Convert result back to protobuf
    pbResult, err := interfaceToProtoValue(result)
    if err != nil {
        return nil, status.Errorf(codes.Internal, "result conversion failed: %v", err)
    }
    
    return &pb.ExecuteResponse{
        Result:  pbResult,
        Success: true,
    }, nil
}

// ExecuteWithAudit implements the ExecuteWithAudit RPC method
func (s *SandboxServiceServer) ExecuteWithAudit(ctx context.Context, req *pb.ExecuteRequest) (*pb.ExecuteWithAuditResponse, error) {
    sb, err := s.pool.Acquire(ctx)
    if err != nil {
        return nil, status.Errorf(codes.Unavailable, "no available sandboxes: %v", err)
    }
    defer s.pool.Release(sb)
    
    args := make([]interface{}, len(req.Args))
    for i, arg := range req.Args {
        args[i] = protoValueToInterface(arg)
    }
    
    auditResult, err := sb.CallWithAudit(ctx, req.Function, args...)
    if err != nil {
        return nil, status.Errorf(codes.Internal, "execution failed: %v", err)
    }
    
    pbResult, err := interfaceToProtoValue(auditResult.Output)
    if err != nil {
        return nil, status.Errorf(codes.Internal, "result conversion failed: %v", err)
    }
    
    // Convert violations and events
    pbViolations := make([]*pb.SecurityViolation, len(auditResult.Violations))
    for i, v := range auditResult.Violations {
        pbViolations[i] = &pb.SecurityViolation{
            Type:        v.Type,
            Description: v.Description,
            Timestamp:   v.Timestamp.Unix(),
        }
    }
    
    pbEvents := make([]*pb.SecurityEvent, len(auditResult.Events))
    for i, e := range auditResult.Events {
        pbEvents[i] = &pb.SecurityEvent{
            Type:        e.Type,
            Description: e.Description,
            Timestamp:   e.Timestamp.Unix(),
        }
    }
    
    return &pb.ExecuteWithAuditResponse{
        Result:     pbResult,
        Success:    true,
        Violations: pbViolations,
        Events:     pbEvents,
        Stats: &pb.ResourceStats{
            CpuTimeMs:       auditResult.Stats.CPUTimeMs,
            PeakMemoryBytes: auditResult.Stats.PeakMemoryBytes,
            FunctionCalls:   auditResult.Stats.FunctionCalls,
        },
    }, nil
}

// Helper functions for protobuf conversion
func protoValueToInterface(pv *pb.Value) interface{} {
    switch v := pv.Kind.(type) {
    case *pb.Value_NumberValue:
        return v.NumberValue
    case *pb.Value_StringValue:
        return v.StringValue
    case *pb.Value_BoolValue:
        return v.BoolValue
    case *pb.Value_ListValue:
        list := make([]interface{}, len(v.ListValue.Values))
        for i, val := range v.ListValue.Values {
            list[i] = protoValueToInterface(val)
        }
        return list
    default:
        return nil
    }
}

func interfaceToProtoValue(v interface{}) (*pb.Value, error) {
    switch val := v.(type) {
    case float64:
        return &pb.Value{Kind: &pb.Value_NumberValue{NumberValue: val}}, nil
    case string:
        return &pb.Value{Kind: &pb.Value_StringValue{StringValue: val}}, nil
    case bool:
        return &pb.Value{Kind: &pb.Value_BoolValue{BoolValue: val}}, nil
    case []interface{}:
        values := make([]*pb.Value, len(val))
        for i, item := range val {
            pbVal, err := interfaceToProtoValue(item)
            if err != nil {
                return nil, err
            }
            values[i] = pbVal
        }
        return &pb.Value{Kind: &pb.Value_ListValue{ListValue: &pb.ListValue{Values: values}}}, nil
    default:
        return &pb.Value{Kind: &pb.Value_StringValue{StringValue: fmt.Sprintf("%v", val)}}, nil
    }
}

func main() {
    ctx := context.Background()
    
    // Create sandbox pool
    pool, err := NewSandboxPool(ctx, 10, "./service-module.wasm")
    if err != nil {
        log.Fatal(err)
    }
    defer pool.Close()
    
    // Create gRPC server
    server := grpc.NewServer()
    service := &SandboxServiceServer{pool: pool}
    pb.RegisterSandboxServiceServer(server, service)
    
    // Listen on port 50051
    listener, err := net.Listen("tcp", ":50051")
    if err != nil {
        log.Fatal(err)
    }
    
    log.Println("gRPC server starting on :50051")
    if err := server.Serve(listener); err != nil {
        log.Fatal(err)
    }
}
```

### Performance Optimization

```go
package main

import (
    "context"
    "fmt"
    "runtime"
    "sync"
    "time"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    "github.com/your-org/wasm-sandbox-go/pkg/cache"
)

// OptimizedSandboxManager provides optimized sandbox management
type OptimizedSandboxManager struct {
    cache       *cache.SandboxCache
    pools       map[string]*SandboxPool
    mu          sync.RWMutex
    warmupSize  int
    maxIdle     time.Duration
}

// NewOptimizedSandboxManager creates a new optimized sandbox manager
func NewOptimizedSandboxManager(options ...ManagerOption) *OptimizedSandboxManager {
    manager := &OptimizedSandboxManager{
        cache:      cache.New(),
        pools:      make(map[string]*SandboxPool),
        warmupSize: runtime.NumCPU(),
        maxIdle:    5 * time.Minute,
    }
    
    for _, opt := range options {
        opt(manager)
    }
    
    return manager
}

// ManagerOption configures the sandbox manager
type ManagerOption func(*OptimizedSandboxManager)

// WithWarmupSize sets the number of sandboxes to pre-warm
func WithWarmupSize(size int) ManagerOption {
    return func(m *OptimizedSandboxManager) {
        m.warmupSize = size
    }
}

// WithMaxIdle sets the maximum idle time before cleanup
func WithMaxIdle(duration time.Duration) ManagerOption {
    return func(m *OptimizedSandboxManager) {
        m.maxIdle = duration
    }
}

// GetOrCreatePool gets or creates a sandbox pool for a module
func (m *OptimizedSandboxManager) GetOrCreatePool(ctx context.Context, moduleID string, source interface{}) (*SandboxPool, error) {
    m.mu.RLock()
    if pool, exists := m.pools[moduleID]; exists {
        m.mu.RUnlock()
        return pool, nil
    }
    m.mu.RUnlock()
    
    m.mu.Lock()
    defer m.mu.Unlock()
    
    // Double-check pattern
    if pool, exists := m.pools[moduleID]; exists {
        return pool, nil
    }
    
    // Create new pool
    pool, err := NewSandboxPool(ctx, m.warmupSize, source)
    if err != nil {
        return nil, err
    }
    
    m.pools[moduleID] = pool
    
    // Start cleanup goroutine for this pool
    go m.cleanupPool(moduleID, pool)
    
    return pool, nil
}

// Execute executes a function using the optimized manager
func (m *OptimizedSandboxManager) Execute(ctx context.Context, moduleID string, source interface{}, function string, args ...interface{}) (interface{}, error) {
    // Check cache first
    if result, found := m.cache.Get(moduleID, function, args); found {
        return result, nil
    }
    
    // Get pool
    pool, err := m.GetOrCreatePool(ctx, moduleID, source)
    if err != nil {
        return nil, err
    }
    
    // Acquire sandbox
    sb, err := pool.Acquire(ctx)
    if err != nil {
        return nil, err
    }
    defer pool.Release(sb)
    
    // Execute function
    result, err := sb.Call(ctx, function, args...)
    if err != nil {
        return nil, err
    }
    
    // Cache result if cacheable
    if m.isCacheable(function, args) {
        m.cache.Set(moduleID, function, args, result, 10*time.Minute)
    }
    
    return result, nil
}

// isCacheable determines if a function result should be cached
func (m *OptimizedSandboxManager) isCacheable(function string, args []interface{}) bool {
    // Simple heuristic: cache pure functions with small args
    if len(args) > 10 {
        return false
    }
    
    // Don't cache functions that might have side effects
    sideEffectFunctions := []string{"write", "delete", "update", "create"}
    for _, fn := range sideEffectFunctions {
        if function == fn {
            return false
        }
    }
    
    return true
}

// cleanupPool manages pool lifecycle
func (m *OptimizedSandboxManager) cleanupPool(moduleID string, pool *SandboxPool) {
    ticker := time.NewTicker(m.maxIdle)
    defer ticker.Stop()
    
    lastUsed := time.Now()
    
    for {
        select {
        case <-ticker.C:
            if time.Since(lastUsed) > m.maxIdle {
                m.mu.Lock()
                delete(m.pools, moduleID)
                m.mu.Unlock()
                
                pool.Close()
                return
            }
        }
    }
}

// Close closes the manager and all pools
func (m *OptimizedSandboxManager) Close() {
    m.mu.Lock()
    defer m.mu.Unlock()
    
    for _, pool := range m.pools {
        pool.Close()
    }
    
    m.pools = make(map[string]*SandboxPool)
}

// Benchmark testing
func BenchmarkSandboxExecution(b *testing.B) {
    ctx := context.Background()
    
    manager := NewOptimizedSandboxManager(
        WithWarmupSize(runtime.NumCPU()),
        WithMaxIdle(1*time.Minute),
    )
    defer manager.Close()
    
    b.ResetTimer()
    
    b.RunParallel(func(pb *testing.PB) {
        for pb.Next() {
            result, err := manager.Execute(ctx, "calc", "./calculator.wasm", "add", 5, 3)
            if err != nil {
                b.Fatal(err)
            }
            
            if result != 8 {
                b.Fatalf("expected 8, got %v", result)
            }
        }
    })
}

// Memory usage monitoring
func (m *OptimizedSandboxManager) GetMemoryStats() map[string]interface{} {
    var ms runtime.MemStats
    runtime.ReadMemStats(&ms)
    
    m.mu.RLock()
    poolCount := len(m.pools)
    m.mu.RUnlock()
    
    return map[string]interface{}{
        "alloc_mb":        ms.Alloc / 1024 / 1024,
        "total_alloc_mb":  ms.TotalAlloc / 1024 / 1024,
        "sys_mb":          ms.Sys / 1024 / 1024,
        "num_gc":          ms.NumGC,
        "active_pools":    poolCount,
        "cache_hits":      m.cache.Hits(),
        "cache_misses":    m.cache.Misses(),
    }
}
```

## Testing

### Test Framework Integration

```go
package sandbox_test

import (
    "context"
    "testing"
    "time"
    
    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
    
    "github.com/your-org/wasm-sandbox-go/pkg/sandbox"
    "github.com/your-org/wasm-sandbox-go/pkg/security"
)

// Test helpers
func createTestSandbox(t *testing.T, source string) sandbox.Sandbox {
    ctx := context.Background()
    
    sb, err := sandbox.NewBuilder().
        Source(source).
        SecurityPolicy(security.PermissivePolicy()). // Relaxed for testing
        Build(ctx)
    
    require.NoError(t, err)
    t.Cleanup(func() { sb.Close() })
    
    return sb
}

func TestBasicFunctionExecution(t *testing.T) {
    sb := createTestSandbox(t, "./test-fixtures/calculator.wasm")
    ctx := context.Background()
    
    tests := []struct {
        name     string
        function string
        args     []interface{}
        expected interface{}
    }{
        {"addition", "add", []interface{}{5, 3}, 8},
        {"multiplication", "multiply", []interface{}{4, 7}, 28},
        {"subtraction", "subtract", []interface{}{10, 4}, 6},
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            result, err := sb.Call(ctx, tt.function, tt.args...)
            require.NoError(t, err)
            assert.Equal(t, tt.expected, result)
        })
    }
}

func TestMemoryLimitEnforcement(t *testing.T) {
    ctx := context.Background()
    
    sb, err := sandbox.NewBuilder().
        Source("./test-fixtures/memory-intensive.wasm").
        MemoryLimit(1024 * 1024). // 1MB limit
        Build(ctx)
    require.NoError(t, err)
    defer sb.Close()
    
    _, err = sb.Call(ctx, "allocate_large_buffer")
    assert.Error(t, err)
    assert.Contains(t, err.Error(), "memory limit exceeded")
}

func TestCPUTimeoutEnforcement(t *testing.T) {
    ctx := context.Background()
    
    sb, err := sandbox.NewBuilder().
        Source("./test-fixtures/cpu-intensive.wasm").
        CPUTimeout(1 * time.Second).
        Build(ctx)
    require.NoError(t, err)
    defer sb.Close()
    
    _, err = sb.Call(ctx, "infinite_loop")
    assert.Error(t, err)
    assert.Contains(t, err.Error(), "timeout exceeded")
}

func TestSecurityPolicyEnforcement(t *testing.T) {
    ctx := context.Background()
    
    sb, err := sandbox.NewBuilder().
        Source("./test-fixtures/network-module.wasm").
        SecurityPolicy(security.StrictPolicy()). // No network access
        Build(ctx)
    require.NoError(t, err)
    defer sb.Close()
    
    _, err = sb.Call(ctx, "make_http_request", "https://example.com")
    assert.Error(t, err)
    
    var secErr *security.ViolationError
    assert.ErrorAs(t, err, &secErr)
    assert.Equal(t, "network_access", secErr.Type)
}

func TestBatchFunctionCalls(t *testing.T) {
    sb := createTestSandbox(t, "./test-fixtures/calculator.wasm")
    ctx := context.Background()
    
    calls := []sandbox.FunctionCall{
        {Function: "add", Args: []interface{}{1, 2}},
        {Function: "multiply", Args: []interface{}{3, 4}},
        {Function: "subtract", Args: []interface{}{10, 5}},
    }
    
    results, err := sb.CallBatch(ctx, calls)
    require.NoError(t, err)
    require.Len(t, results, 3)
    
    assert.Equal(t, 3, results[0])  // 1 + 2
    assert.Equal(t, 12, results[1]) // 3 * 4
    assert.Equal(t, 5, results[2])  // 10 - 5
}

func TestResourceMonitoring(t *testing.T) {
    sb := createTestSandbox(t, "./test-fixtures/calculator.wasm")
    ctx := context.Background()
    
    // Initial state
    initialStats, err := sb.ResourceStats(ctx)
    require.NoError(t, err)
    assert.Equal(t, int64(0), initialStats.FunctionCalls)
    
    // Execute some functions
    _, err = sb.Call(ctx, "add", 1, 2)
    require.NoError(t, err)
    
    _, err = sb.Call(ctx, "multiply", 3, 4)
    require.NoError(t, err)
    
    // Check updated stats
    updatedStats, err := sb.ResourceStats(ctx)
    require.NoError(t, err)
    assert.Equal(t, int64(2), updatedStats.FunctionCalls)
    assert.Greater(t, updatedStats.CPUTimeMs, initialStats.CPUTimeMs)
}

func TestSandboxReset(t *testing.T) {
    sb := createTestSandbox(t, "./test-fixtures/stateful-module.wasm")
    ctx := context.Background()
    
    // Set some state
    _, err := sb.Call(ctx, "set_state", 42)
    require.NoError(t, err)
    
    state, err := sb.Call(ctx, "get_state")
    require.NoError(t, err)
    assert.Equal(t, 42, state)
    
    // Reset sandbox
    err = sb.Reset(ctx)
    require.NoError(t, err)
    
    // State should be cleared
    state, err = sb.Call(ctx, "get_state")
    require.NoError(t, err)
    assert.Equal(t, 0, state) // Default state
}

// Benchmark tests
func BenchmarkSandboxCreation(b *testing.B) {
    ctx := context.Background()
    
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        sb, err := sandbox.FromFile(ctx, "./test-fixtures/calculator.wasm")
        if err != nil {
            b.Fatal(err)
        }
        sb.Close()
    }
}

func BenchmarkFunctionExecution(b *testing.B) {
    sb := createTestSandbox(b, "./test-fixtures/calculator.wasm")
    ctx := context.Background()
    
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        _, err := sb.Call(ctx, "add", 5, 3)
        if err != nil {
            b.Fatal(err)
        }
    }
}

func BenchmarkConcurrentExecution(b *testing.B) {
    sb := createTestSandbox(b, "./test-fixtures/calculator.wasm")
    ctx := context.Background()
    
    b.ResetTimer()
    
    b.RunParallel(func(pb *testing.PB) {
        for pb.Next() {
            _, err := sb.Call(ctx, "add", 5, 3)
            if err != nil {
                b.Fatal(err)
            }
        }
    })
}
```

## Examples

### Real-world Usage Examples

Check the [`examples/`](./examples/) directory for complete working examples:

- **[HTTP Server](./examples/http-server/)** - High-performance HTTP server with sandbox processing
- **[gRPC Service](./examples/grpc-service/)** - gRPC service with WebAssembly computation
- **[CLI Tool](./examples/cli-tool/)** - Command-line application with plugin support
- **[Microservice](./examples/microservice/)** - Containerized Go microservice
- **[Worker Pool](./examples/worker-pool/)** - Distributed processing with worker goroutines
- **[Kubernetes Operator](./examples/k8s-operator/)** - Kubernetes operator for WASM workloads

---

**Go Excellence**: Production-ready Go bindings with idiomatic APIs, comprehensive concurrency support, and enterprise-grade performance optimization.
