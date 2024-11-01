use oxitty::{AtomicState, StateSnapshot};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Instant,
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(Debug, Default)]
struct MemoryMetrics {
    total_bytes: u64,
    current_bytes: usize,
    total_blocks: u64,
    current_blocks: usize,
    peak_bytes: usize,
    peak_blocks: usize,
}

impl MemoryMetrics {
    fn from_heap_stats(stats: &dhat::HeapStats) -> Self {
        Self {
            total_bytes: stats.total_bytes,
            current_bytes: stats.curr_bytes,
            total_blocks: stats.total_blocks,
            current_blocks: stats.curr_blocks,
            peak_bytes: stats.max_bytes,
            peak_blocks: stats.max_blocks,
        }
    }

    fn diff(&self, other: &Self) -> Self {
        Self {
            total_bytes: self.total_bytes.saturating_sub(other.total_bytes),
            current_bytes: self.current_bytes.saturating_sub(other.current_bytes),
            total_blocks: self.total_blocks.saturating_sub(other.total_blocks),
            current_blocks: self.current_blocks.saturating_sub(other.current_blocks),
            peak_bytes: self.peak_bytes.max(other.peak_bytes),
            peak_blocks: self.peak_blocks.max(other.peak_blocks),
        }
    }
}

#[derive(Debug)]
struct MemState {
    data: Vec<Vec<u8>>, // Vector of vectors for more allocation activity
    running: AtomicBool,
}

#[derive(Debug, Clone)]
struct MemSnapshot {
    running: bool,
}

impl StateSnapshot for MemSnapshot {
    fn should_quit(&self) -> bool {
        !self.running
    }
}

impl AtomicState for MemState {
    type Snapshot = MemSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        MemSnapshot {
            running: self.running.load(Ordering::Acquire),
        }
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    fn quit(&self) {
        self.running.store(false, Ordering::Release)
    }
}

fn apply_memory_pressure(state: &mut MemState) {
    // Create multiple allocations of varying sizes
    for _ in 0..10 {
        let size = (rand::random::<usize>() % 1000) + 100;
        let mut vec = Vec::with_capacity(size);
        vec.extend((0..size).map(|i| i as u8));
        state.data.push(vec);
    }

    // Clear half the vectors to create churn
    if state.data.len() > 5 {
        state.data.drain(0..state.data.len() / 2);
    }

    thread::sleep(std::time::Duration::from_millis(1));
}

fn print_metrics(name: &str, metrics: &MemoryMetrics) {
    println!("\n=== {} ===", name);
    println!(
        "Total allocated: {} bytes ({} blocks)",
        metrics.total_bytes, metrics.total_blocks
    );
    println!(
        "Current memory: {} bytes ({} blocks)",
        metrics.current_bytes, metrics.current_blocks
    );
    println!(
        "Peak memory: {} bytes ({} blocks)",
        metrics.peak_bytes, metrics.peak_blocks
    );
    println!(
        "Memory utilization: {:.2}%",
        if metrics.peak_bytes > 0 {
            (metrics.current_bytes as f64 / metrics.peak_bytes as f64) * 100.0
        } else {
            0.0
        }
    );
}

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    println!("🔍 Starting memory profile analysis...\n");

    let baseline = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());
    print_metrics("Baseline", &baseline);

    let mut time_series = Vec::new();
    let start_time = Instant::now();

    for size in [100, 1000, 10000] {
        println!("\n📊 Testing state size: {}", size);

        let before_state = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());
        let mut state = MemState {
            data: Vec::with_capacity(size),
            running: AtomicBool::new(true),
        };
        // Initial allocation
        for i in 0..size {
            state.data.push(vec![i as u8; i % 100 + 1]);
        }
        let after_state = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());

        let state_impact = after_state.diff(&before_state);
        println!("\n🔧 State Creation Impact:");
        println!(
            "Memory per item: {:.2} bytes",
            state_impact.total_bytes as f64 / size as f64
        );

        let mut snapshot_sizes = Vec::new();
        let mut peak_usage = 0;

        for i in 0..50 {
            // Reduced iterations but more intense
            if i % 5 == 0 {
                // More frequent pressure
                apply_memory_pressure(&mut state);
            }

            let before_snap = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());
            let _snapshot = state.snapshot();
            let after_snap = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());

            let snap_diff = after_snap.diff(&before_snap);
            snapshot_sizes.push(snap_diff.total_bytes);
            peak_usage = peak_usage.max(after_snap.current_bytes);

            time_series.push((
                start_time.elapsed().as_secs_f64(),
                after_snap.current_bytes as f64 / 1024.0,
                after_snap.current_bytes as f64 / after_snap.peak_bytes as f64 * 100.0,
            ));

            if i % 10 == 0 {
                // Periodic cleanup
                state.data.truncate(state.data.len() / 2);
            }
        }

        println!("\n📈 Memory Analysis:");
        println!(
            "Average snapshot cost: {:.2} bytes",
            snapshot_sizes.iter().sum::<u64>() as f64 / snapshot_sizes.len() as f64
        );
        println!("Peak memory usage: {} bytes", peak_usage);
        println!("Memory variance: {:.2}", {
            let mean = snapshot_sizes.iter().sum::<u64>() as f64 / snapshot_sizes.len() as f64;
            let variance = snapshot_sizes
                .iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum::<f64>()
                / snapshot_sizes.len() as f64;
            variance.sqrt()
        });

        let final_metrics = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());
        println!("\n🔎 Memory Status:");
        println!("Live allocations: {} blocks", final_metrics.current_blocks);
        println!(
            "Memory utilization: {:.2}%",
            (final_metrics.current_bytes as f64 / final_metrics.peak_bytes as f64) * 100.0
        );
    }

    // Memory timeline visualization
    println!("\n📊 Memory Usage Timeline:");
    let max_memory = time_series
        .iter()
        .map(|&(_, mem, _)| mem)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);
    let mut last_time = 0.0;

    for (time, mem_kb, util) in time_series.iter().step_by(5) {
        // Sample every 5th point
        if *time - last_time >= 0.1 {
            let bar_length = (40.0 * mem_kb / max_memory) as usize;
            println!(
                "{:>6.2}s |{:=<width$}| {:.1} KB ({:.1}% util)",
                time,
                "",
                mem_kb,
                util,
                width = bar_length
            );
            last_time = *time;
        }
    }

    let final_metrics = MemoryMetrics::from_heap_stats(&dhat::HeapStats::get());
    print_metrics("Final Metrics", &final_metrics);
}
