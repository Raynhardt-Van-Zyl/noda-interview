use std::time::{Duration, Instant};

/// Runtime counters collected during one CLI run.
#[derive(Debug)]
pub struct RunMetrics {
    started_at: Instant,
    pub total_records: usize,
    pub successful_rows: usize,
    pub failed_rows: usize,
    pub filtered_empty_tags: usize,
}

impl RunMetrics {
    /// Start a new metrics timer with zeroed counters.
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
            total_records: 0,
            successful_rows: 0,
            failed_rows: 0,
            filtered_empty_tags: 0,
        }
    }

    /// Wall-clock time since the run started.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Throughput based on all raw records read from input.
    pub fn rows_per_second(&self) -> f64 {
        let seconds = self.elapsed().as_secs_f64();
        if seconds == 0.0 {
            return 0.0;
        }

        self.total_records as f64 / seconds
    }

    /// Human-readable summary printed by the CLI.
    pub fn summary(&self) -> String {
        format!(
            "Total records processed: {}\nSuccessful rows written: {}\nFailed rows: {}\nFiltered empty tags: {}\nTotal duration: {:.3}s\nRows per second: {:.2}",
            self.total_records,
            self.successful_rows,
            self.failed_rows,
            self.filtered_empty_tags,
            self.elapsed().as_secs_f64(),
            self.rows_per_second()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_summary_with_all_counters() {
        let mut metrics = RunMetrics::start();
        metrics.total_records = 3;
        metrics.successful_rows = 1;
        metrics.failed_rows = 1;
        metrics.filtered_empty_tags = 1;

        let summary = metrics.summary();

        assert!(summary.contains("Total records processed: 3"));
        assert!(summary.contains("Successful rows written: 1"));
        assert!(summary.contains("Failed rows: 1"));
        assert!(summary.contains("Filtered empty tags: 1"));
    }
}
