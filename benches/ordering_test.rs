use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

// Simulates Oxitty's state management pattern
struct StateTest {
    running: AtomicBool,
    value: AtomicU64,
    ready: AtomicBool,
}

impl StateTest {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            value: AtomicU64::new(0),
            ready: AtomicBool::new(false),
        }
    }
}

fn bench_orderings(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_orderings");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    // Simulates Oxitty's event passing system
    group.bench_function("event_channel_relaxed", |b| {
        let state = Arc::new(StateTest::new());
        let state_clone = state.clone();

        let producer = thread::spawn(move || {
            while state_clone.running.load(Ordering::Relaxed) {
                state_clone.value.fetch_add(1, Ordering::Relaxed);
                state_clone.ready.store(true, Ordering::Relaxed);
            }
        });

        b.iter(|| {
            if state.ready.load(Ordering::Relaxed) {
                black_box(state.value.load(Ordering::Relaxed));
                state.ready.store(false, Ordering::Relaxed);
            }
        });

        state.running.store(false, Ordering::Relaxed);
        producer.join().unwrap();
    });

    // Simulates Oxitty's event passing with proper ordering
    group.bench_function("event_channel_ordered", |b| {
        let state = Arc::new(StateTest::new());
        let state_clone = state.clone();

        let producer = thread::spawn(move || {
            while state_clone.running.load(Ordering::Acquire) {
                state_clone.value.fetch_add(1, Ordering::Release);
                state_clone.ready.store(true, Ordering::Release);
            }
        });

        b.iter(|| {
            if state.ready.load(Ordering::Acquire) {
                black_box(state.value.load(Ordering::Acquire));
                state.ready.store(false, Ordering::Release);
            }
        });

        state.running.store(false, Ordering::Release);
        producer.join().unwrap();
    });

    // Simulates Oxitty's snapshot system
    group.bench_function("snapshot_relaxed", |b| {
        let state = Arc::new(StateTest::new());

        b.iter(|| {
            let running = state.running.load(Ordering::Relaxed);
            let value = state.value.load(Ordering::Relaxed);
            black_box((running, value))
        });
    });

    group.bench_function("snapshot_acquire", |b| {
        let state = Arc::new(StateTest::new());

        b.iter(|| {
            let running = state.running.load(Ordering::Acquire);
            let value = state.value.load(Ordering::Acquire);
            black_box((running, value))
        });
    });

    // Simulates state updates
    group.bench_function("state_update_relaxed", |b| {
        let state = Arc::new(StateTest::new());
        let state_clone = state.clone();

        let updater = thread::spawn(move || {
            while state_clone.running.load(Ordering::Relaxed) {
                state_clone.value.fetch_add(1, Ordering::Relaxed);
                thread::yield_now();
            }
        });

        b.iter(|| {
            state.value.fetch_add(1, Ordering::Relaxed);
            black_box(state.value.load(Ordering::Relaxed))
        });

        state.running.store(false, Ordering::Relaxed);
        updater.join().unwrap();
    });

    group.bench_function("state_update_seqcst", |b| {
        let state = Arc::new(StateTest::new());
        let state_clone = state.clone();

        let updater = thread::spawn(move || {
            while state_clone.running.load(Ordering::SeqCst) {
                state_clone.value.fetch_add(1, Ordering::SeqCst);
                thread::yield_now();
            }
        });

        b.iter(|| {
            state.value.fetch_add(1, Ordering::SeqCst);
            black_box(state.value.load(Ordering::SeqCst))
        });

        state.running.store(false, Ordering::SeqCst);
        updater.join().unwrap();
    });

    group.finish();
}

criterion_group!(
    name = ordering_benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10));
    targets = bench_orderings
);
criterion_main!(ordering_benches);
