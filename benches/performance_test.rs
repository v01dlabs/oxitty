use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use oxitty::{App, AtomicState, Event, StateSnapshot};
use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

// Test state for benchmarking
#[derive(Debug)]
struct BenchState {
    counters: Vec<AtomicU64>,
    running: AtomicBool,
}

impl BenchState {
    fn new(size: usize) -> Self {
        Self {
            counters: (0..size).map(|_| AtomicU64::new(0)).collect(),
            running: AtomicBool::new(true),
        }
    }

    fn increment_all(&self) {
        for counter in &self.counters {
            counter.fetch_add(1, Ordering::Release);
        }
    }
}

#[derive(Debug, Clone)]
struct BenchSnapshot {
    running: bool,
}

impl StateSnapshot for BenchSnapshot {
    fn should_quit(&self) -> bool {
        !self.running
    }
}

impl AtomicState for BenchState {
    type Snapshot = BenchSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        BenchSnapshot {
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

pub fn bench_state_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_updates");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("atomic_increments", size),
            size,
            |b, &size| {
                let state = BenchState::new(size);
                b.iter(|| {
                    black_box(&state).increment_all();
                });
            },
        );
    }
    group.finish();
}

pub fn bench_snapshots(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshots");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("state_snapshot", size),
            size,
            |b, &size| {
                let state = BenchState::new(size);
                b.iter(|| {
                    black_box(&state).snapshot();
                });
            },
        );
    }
    group.finish();
}

pub fn bench_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_handling");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("event_send_recv", |b| {
        smol::block_on(async {
            let state = BenchState::new(100);
            let app = App::new(state, Duration::from_millis(50)).unwrap();
            let events = app.events();

            b.iter(|| {
                let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());

                black_box(&events).try_send(Event::Key(key_event)).unwrap();

                while black_box(&events).try_recv().unwrap().is_some() {}
            });
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10));
    targets = bench_state_updates, bench_snapshots, bench_events
);
criterion_main!(benches);
