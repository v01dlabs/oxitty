use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oxitty::{AtomicState, StateSnapshot};
use rayon::prelude::*;
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

// Test state for parallel benchmarking
#[derive(Debug)]
struct ParallelBenchState {
    counters: Vec<AtomicU64>,
    running: AtomicBool,
}

impl ParallelBenchState {
    fn new(size: usize) -> Self {
        Self {
            counters: (0..size).map(|_| AtomicU64::new(0)).collect(),
            running: AtomicBool::new(true),
        }
    }

    fn increment_all_sequential(&self) {
        for counter in &self.counters {
            counter.fetch_add(1, Ordering::Release);
        }
    }

    fn increment_all_parallel(&self) {
        self.counters.par_iter().for_each(|counter| {
            counter.fetch_add(1, Ordering::Release);
        });
    }

    // Batch processing for better parallel efficiency
    fn increment_all_parallel_chunked(&self) {
        self.counters.par_chunks(128).for_each(|chunk| {
            chunk.iter().for_each(|counter| {
                counter.fetch_add(1, Ordering::Release);
            });
        });
    }
}

#[derive(Debug, Clone)]
struct ParallelBenchSnapshot {
    running: bool,
}

impl StateSnapshot for ParallelBenchSnapshot {
    fn should_quit(&self) -> bool {
        !self.running
    }
}

impl AtomicState for ParallelBenchState {
    type Snapshot = ParallelBenchSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        ParallelBenchSnapshot {
            running: self.running.load(Ordering::Acquire),
        }
    }

    fn quit(&self) {
        self.running.store(false, Ordering::Release);
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

pub fn bench_parallel_state_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_state_updates");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Test different sizes to see scaling characteristics
    for size in [100, 1000, 10000, 100000].iter() {
        // Sequential updates
        group.bench_with_input(
            BenchmarkId::new("sequential", size),
            size,
            |b, &size| {
                let state = ParallelBenchState::new(size);
                b.iter(|| {
                    black_box(&state).increment_all_sequential();
                });
            },
        );

        // Parallel updates
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            size,
            |b, &size| {
                let state = ParallelBenchState::new(size);
                b.iter(|| {
                    black_box(&state).increment_all_parallel();
                });
            },
        );

        // Parallel chunked updates
        group.bench_with_input(
            BenchmarkId::new("parallel_chunked", size),
            size,
            |b, &size| {
                let state = ParallelBenchState::new(size);
                b.iter(|| {
                    black_box(&state).increment_all_parallel_chunked();
                });
            },
        );
    }
    group.finish();
}

pub fn bench_parallel_snapshots(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_snapshots");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000, 100000].iter() {
        // Sequential snapshots
        group.bench_with_input(
            BenchmarkId::new("sequential_snapshot", size),
            size,
            |b, &size| {
                let state = ParallelBenchState::new(size);
                b.iter(|| {
                    black_box(&state).snapshot();
                });
            },
        );

        // Parallel snapshot creation and processing
        group.bench_with_input(
            BenchmarkId::new("parallel_snapshot_processing", size),
            size,
            |b, &size| {
                let state = ParallelBenchState::new(size);
                b.iter(|| {
                    rayon::scope(|s| {
                        for _ in 0..4 {
                            s.spawn(|_| {
                                black_box(&state).snapshot();
                            });
                        }
                    });
                });
            },
        );
    }
    group.finish();
}

// Configure and run benchmarks
criterion_group!(
    name = parallel_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .noise_threshold(0.05);
    targets = bench_parallel_state_updates, bench_parallel_snapshots
);
criterion_main!(parallel_benches);
