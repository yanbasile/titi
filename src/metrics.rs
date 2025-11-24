use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub grid_memory_bytes: usize,
    pub atlas_memory_bytes: usize,
    pub total_allocations: usize,
    pub peak_memory_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub frame_time_ms: f64,
    pub fps: f64,
    pub parse_time_ms: f64,
    pub render_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct TerminalMetrics {
    pub pane_id: String,
    pub grid_size: (usize, usize),
    pub cell_count: usize,
    pub memory_bytes: usize,
    pub last_update: Instant,
    pub total_writes: usize,
    pub total_bytes_written: usize,
}

pub struct MetricsCollector {
    memory: Arc<Mutex<MemoryMetrics>>,
    performance: Arc<Mutex<PerformanceMetrics>>,
    terminals: Arc<Mutex<HashMap<String, TerminalMetrics>>>,
    frame_times: Arc<Mutex<Vec<Duration>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(MemoryMetrics {
                grid_memory_bytes: 0,
                atlas_memory_bytes: 0,
                total_allocations: 0,
                peak_memory_bytes: 0,
            })),
            performance: Arc::new(Mutex::new(PerformanceMetrics {
                frame_time_ms: 0.0,
                fps: 0.0,
                parse_time_ms: 0.0,
                render_time_ms: 0.0,
            })),
            terminals: Arc::new(Mutex::new(HashMap::new())),
            frame_times: Arc::new(Mutex::new(Vec::with_capacity(60))),
            start_time: Instant::now(),
        }
    }

    pub fn record_frame(&self, duration: Duration) {
        let mut frame_times = self.frame_times.lock().unwrap();
        frame_times.push(duration);

        // Keep only last 60 frames
        if frame_times.len() > 60 {
            frame_times.remove(0);
        }

        // Calculate average FPS
        if !frame_times.is_empty() {
            let avg_frame_time: Duration = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
            let fps = 1.0 / avg_frame_time.as_secs_f64();

            let mut perf = self.performance.lock().unwrap();
            perf.frame_time_ms = avg_frame_time.as_secs_f64() * 1000.0;
            perf.fps = fps;
        }
    }

    pub fn record_parse_time(&self, duration: Duration) {
        let mut perf = self.performance.lock().unwrap();
        perf.parse_time_ms = duration.as_secs_f64() * 1000.0;
    }

    pub fn record_render_time(&self, duration: Duration) {
        let mut perf = self.performance.lock().unwrap();
        perf.render_time_ms = duration.as_secs_f64() * 1000.0;
    }

    pub fn update_memory(&self, grid_bytes: usize, atlas_bytes: usize) {
        let mut mem = self.memory.lock().unwrap();
        mem.grid_memory_bytes = grid_bytes;
        mem.atlas_memory_bytes = atlas_bytes;

        let total = grid_bytes + atlas_bytes;
        if total > mem.peak_memory_bytes {
            mem.peak_memory_bytes = total;
        }
    }

    pub fn register_terminal(&self, pane_id: String, cols: usize, rows: usize) {
        let mut terminals = self.terminals.lock().unwrap();

        let cell_count = cols * rows;
        let memory_bytes = cell_count * std::mem::size_of::<crate::terminal::Cell>();

        terminals.insert(
            pane_id.clone(),
            TerminalMetrics {
                pane_id,
                grid_size: (cols, rows),
                cell_count,
                memory_bytes,
                last_update: Instant::now(),
                total_writes: 0,
                total_bytes_written: 0,
            },
        );

        log::debug!("Registered terminal: {} cells, {} bytes", cell_count, memory_bytes);
    }

    pub fn record_terminal_write(&self, pane_id: &str, bytes: usize) {
        let mut terminals = self.terminals.lock().unwrap();

        if let Some(metrics) = terminals.get_mut(pane_id) {
            metrics.total_writes += 1;
            metrics.total_bytes_written += bytes;
            metrics.last_update = Instant::now();
        }
    }

    pub fn unregister_terminal(&self, pane_id: &str) {
        let mut terminals = self.terminals.lock().unwrap();

        if let Some(metrics) = terminals.remove(pane_id) {
            log::info!(
                "Unregistered terminal {}: {} writes, {} bytes written, {} bytes freed",
                pane_id,
                metrics.total_writes,
                metrics.total_bytes_written,
                metrics.memory_bytes
            );
        }
    }

    pub fn detect_memory_leaks(&self) -> Vec<String> {
        let terminals = self.terminals.lock().unwrap();
        let now = Instant::now();
        let mut leaks = Vec::new();

        for (pane_id, metrics) in terminals.iter() {
            // If terminal hasn't been updated in 5 minutes, it might be leaked
            if now.duration_since(metrics.last_update) > Duration::from_secs(300) {
                leaks.push(format!(
                    "Terminal {} inactive for {} seconds, {} bytes",
                    pane_id,
                    now.duration_since(metrics.last_update).as_secs(),
                    metrics.memory_bytes
                ));
            }
        }

        leaks
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        self.memory.lock().unwrap().clone()
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance.lock().unwrap().clone()
    }

    pub fn get_terminal_metrics(&self) -> HashMap<String, TerminalMetrics> {
        self.terminals.lock().unwrap().clone()
    }

    pub fn print_summary(&self) {
        let mem = self.memory.lock().unwrap();
        let perf = self.performance.lock().unwrap();
        let terminals = self.terminals.lock().unwrap();

        let uptime = Instant::now().duration_since(self.start_time);

        log::info!("╔════════════════════════════════════════╗");
        log::info!("║       METRICS SUMMARY                  ║");
        log::info!("╠════════════════════════════════════════╣");
        log::info!("║ Uptime: {:?}", uptime);
        log::info!("║");
        log::info!("║ PERFORMANCE:");
        log::info!("║   FPS: {:.2}", perf.fps);
        log::info!("║   Frame Time: {:.2}ms", perf.frame_time_ms);
        log::info!("║   Parse Time: {:.2}ms", perf.parse_time_ms);
        log::info!("║   Render Time: {:.2}ms", perf.render_time_ms);
        log::info!("║");
        log::info!("║ MEMORY:");
        log::info!("║   Grid Memory: {} KB", mem.grid_memory_bytes / 1024);
        log::info!("║   Atlas Memory: {} KB", mem.atlas_memory_bytes / 1024);
        log::info!("║   Peak Memory: {} KB", mem.peak_memory_bytes / 1024);
        log::info!("║");
        log::info!("║ TERMINALS:");
        log::info!("║   Active Panes: {}", terminals.len());

        for (pane_id, metrics) in terminals.iter() {
            log::info!("║   - {}: {}x{} ({} bytes, {} writes)",
                      pane_id,
                      metrics.grid_size.0,
                      metrics.grid_size.1,
                      metrics.memory_bytes,
                      metrics.total_writes);
        }

        log::info!("╚════════════════════════════════════════╝");
    }

    pub fn log_memory_warning(&self) {
        let mem = self.memory.lock().unwrap();
        let total_mb = (mem.grid_memory_bytes + mem.atlas_memory_bytes) as f64 / (1024.0 * 1024.0);

        if total_mb > 100.0 {
            log::warn!("HIGH MEMORY USAGE: {:.2} MB", total_mb);
        }

        // Check for memory leaks
        drop(mem);
        let leaks = self.detect_memory_leaks();

        if !leaks.is_empty() {
            log::warn!("POTENTIAL MEMORY LEAKS DETECTED:");
            for leak in leaks {
                log::warn!("  - {}", leak);
            }
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Global metrics instance
lazy_static::lazy_static! {
    pub static ref METRICS: MetricsCollector = MetricsCollector::new();
}

// Convenience macros
#[macro_export]
macro_rules! record_frame {
    ($duration:expr) => {
        $crate::metrics::METRICS.record_frame($duration);
    };
}

#[macro_export]
macro_rules! record_parse_time {
    ($duration:expr) => {
        $crate::metrics::METRICS.record_parse_time($duration);
    };
}

#[macro_export]
macro_rules! record_render_time {
    ($duration:expr) => {
        $crate::metrics::METRICS.record_render_time($duration);
    };
}
