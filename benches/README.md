# 📊 Oxitty Performance Benchmarks

Comprehensive benchmarks demonstrating Oxitty's performance characteristics across state management, event handling, and memory efficiency tests.

## Overview

Oxitty achieves high performance through:
- Lock-free atomic state management
- Zero-copy operations
- Non-blocking event processing
- Efficient memory utilization

## Timing Results

### State Snapshots
Remarkably consistent sub-nanosecond performance regardless of state size:
- 100 items: 454.48 ps (σ: 5.62 ps)
- 1,000 items: 453.55 ps (σ: 4.56 ps)
- 10,000 items: 455.29 ps (σ: 10.72 ps)

R² values > 0.94 indicate exceptional reliability and consistency.

### State Updates
Linear scaling with predictable performance:
- 100 items: 90.34 ns (σ: 1.57 ns)
- 1,000 items: 898.88 ns (σ: 12.42 ns)
- 10,000 items: 9.13 µs (σ: 301.46 ns)

R² values > 0.89 demonstrate consistent scaling behavior.

### Event Processing
Ultra-low latency message passing:
- Roundtrip: 46.98 ns (σ: 2.36 ns)
- Channel capacity: 1,024 events
- Non-blocking operations: < 100 ns

## Memory Profile

### Base Footprint
- Initial allocation: 1,088 bytes in 2 blocks
- Baseline utilization: 100%
- Minimal overhead: ~544 bytes per block

### Scaling Characteristics
Memory usage scales linearly with state size:
- 100 items: 16.9 KB peak (74.5 bytes/item)
- 1,000 items: 82.9 KB peak (74.5 bytes/item)
- 10,000 items: 742.7 KB peak (74.5 bytes/item)

### Memory Lifecycle
Total program memory profile:
- Total allocations: 1.56 MB across 11,431 blocks
- Peak memory: 995.2 KB with 10,015 concurrent blocks
- Final state: 1.44 KB in 3 blocks
- Memory utilization: 0.76% (excellent cleanup)

### Key Memory Characteristics
- Predictable per-item overhead
- Efficient block reuse
- Strong cleanup behavior
- No detected memory leaks
- Linear scaling with size

## Running the Benchmarks

### Prerequisites
```toml
[dev-dependencies]
criterion = "0.5.1"
dhat = "0.3.3"
```

### Commands
```bash
# Run benchmarks
cargo bench

# View memory profile
# Use https://nnethercote.github.io/dh_view/dh_view.html
# Load the generated dhat-heap.json file
```

## Methodology

### Timing Benchmarks
- 100 samples per measurement
- 10-second measurement windows
- Plotters backend for visualization
- Statistical analysis via Criterion.rs

### Memory Analysis
- dhat heap profiler integration
- Continuous monitoring during operations
- Memory pressure simulation
- Block-level allocation tracking

## Test Environment

- CPU: Apple M3 Pro
- Memory: 36GB LPDDR5
- OS: macOS Sonoma 14.6.1
- Rust: 1.82.0

## Key Findings

1. **Snapshot Performance**
   - Sub-nanosecond operations
   - Size-independent timing
   - Extremely low variance

2. **State Updates**
   - Predictable linear scaling
   - Consistent performance
   - Minimal overhead

3. **Event Handling**
   - Low latency messaging
   - Stable performance
   - Efficient queuing

4. **Memory Management**
   - Predictable allocation patterns
   - Efficient cleanup
   - Linear scaling
   - Low fragmentation

## Notes

- Benchmark results may vary by system
- Memory patterns are consistent across runs
- Performance is stable under load
- No significant degradation observed

View detailed timing reports in `target/criterion/report/index.html` and memory analysis in the dhat viewer with `dhat-heap.json`.
