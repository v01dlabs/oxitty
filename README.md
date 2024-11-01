# 🐱 Oxitty

A cohesive, opinionated framework that combines well-known libraries ([crossterm](https://github.com/crossterm-rs/crossterm), [ratatui](https://github.com/ratatui/ratatui), [smol](https://github.com/smol-rs/smol)) into one for building consistent terminal applications.

## 🎯 Project Goals

1. **Performance**: Maintain sub-nanosecond state snapshots and minimal memory overhead
2. **Safety**: Zero unsafe code, thread-safe operations, proper error handling
3. **Ergonomics**: Intuitive API design, clear documentation
4. **Modularity**: Clean separation of concerns, flexible architecture
5. **Aesthetics**: Pretty terminal UIs with expressive theming

## 💡 Design Philosophy

Oxitty embraces several key design principles:

1. **Zero-Cost Abstractions**
   - Atomic operations over locks
   - Static dispatch over dynamic
   - Stack allocation when possible

2. **Type Safety**
   - Strong type system usage
   - Compile-time guarantees
   - No unsafe code

3. **Efficient Memory Use**
   - Predictable allocation patterns
   - Zero-copy operations
   - Bounded channels

4. **Clean Architecture**
   - Clear separation of concerns
   - Modular component design
   - Flexible extension points

## 🎯 Architecture

1. **Atomic State Management**
   - Thread-safe state mutations
   - Immutable snapshots for consistency
   - Zero-cost state updates
   - Type-safe state access

2. **Event System**
   - Non-blocking event processing
   - Bounded channels (1024 events)
   - Custom event support
   - Zero-copy message passing

3. **Color System**
   - Full RGBA color support
   - HSL/RGB/Hex conversions
   - Theme management
   - Efficient color operations

4. **Terminal Management**
   - Raw mode handling
   - Alternate screen support
   - Mouse capture
   - Clean shutdown

4. **Error Handling**
   - Rich diagnostic information
   - Source code context
   - Error spans
   - Clean error types

## 📊 Performance

### State Management
```plaintext
State Snapshots (100-10k items)
├── Time: 454.48ps - 455.29ps
├── Variance: σ < 11ps
└── R² values > 0.94

State Updates
├── 100 items: 90.34ns
├── 1k items: 898.88ns
└── 10k items: 9.13μs
```

### Memory Profile
```plaintext
Base Footprint
├── Initial: 1,088 bytes
├── Per-item: 74.5 bytes
└── Utilization: ~100%

Scaling (Peak Memory)
├── 100 items: 16.9 KB
├── 1k items: 82.9 KB
└── 10k items: 742.7 KB
```

### Event System
```plaintext
Event Processing
├── Roundtrip: 46.98ns
├── Variance: σ = 2.36ns
└── Channel capacity: 1,024
```

## 🛠️ Development

```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Build docs
cargo doc --no-deps --open
```

## 📄 License

This project is licensed under the Mozilla Public License 2.0 - see the [LICENSE](LICENSE) file for details.

---

Made with ♥️ by a terminal enthusiast
