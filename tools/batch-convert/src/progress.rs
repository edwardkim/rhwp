use std::time::Instant;

/// Progress tracking for batch conversions
pub struct ProgressTracker {
    pub total: usize,
    pub current: usize,
    pub started_at: Instant,
}

impl ProgressTracker {
    pub fn new(total: usize) -> Self {
        ProgressTracker {
            total,
            current: 0,
            started_at: Instant::now(),
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64()
    }

    pub fn estimated_remaining_seconds(&self) -> f64 {
        if self.current == 0 {
            0.0
        } else {
            let elapsed = self.elapsed_seconds();
            let per_file = elapsed / self.current as f64;
            per_file * (self.total - self.current) as f64
        }
    }

    pub fn format_time(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u64;
        let minutes = ((seconds % 3600.0) / 60.0) as u64;
        let secs = (seconds % 60.0) as u64;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }

    pub fn status_line(&self) -> String {
        let elapsed = Self::format_time(self.elapsed_seconds());
        let remaining = Self::format_time(self.estimated_remaining_seconds());
        let percentage = self.percentage();

        format!(
            "[{:3.0}%] {}/{} files | Elapsed: {} | ETA: {}",
            percentage, self.current, self.total, elapsed, remaining
        )
    }
}
