//! Runtime counters returned by [`crate::run_etl`].
//!
//! Metrics are intentionally aggregate-only. Use [`crate::event_log`] when a
//! caller needs row-level failure details.

use std::time::{Duration, Instant};

/// Runtime counters collected during one ETL run.
///
/// `RunMetrics` is returned by [`crate::run_etl`] after the run has completed.
/// At that point the elapsed duration is frozen, so later calls to
/// [`elapsed`](Self::elapsed), [`rows_per_second`](Self::rows_per_second), or
/// [`summary`](Self::summary) describe the completed ETL run rather than the
/// wall-clock time since the struct was created.
#[derive(Debug)]
pub struct RunMetrics {
    started_at: Instant,
    completed_in: Option<Duration>,

    /// Number of physical input records read.
    pub total_records: usize,

    /// Number of rows inserted into SQLite.
    pub successful_rows: usize,

    /// Number of malformed, invalid, or database-rejected rows.
    pub failed_rows: usize,

    /// Number of otherwise valid rows skipped because their tag normalized to empty.
    pub filtered_empty_tags: usize,
}

impl RunMetrics {
    /// Start a new metrics timer with zeroed counters.
    pub fn start() -> Self {
        Self {
            started_at: Instant::now(),
            completed_in: None,
            total_records: 0,
            successful_rows: 0,
            failed_rows: 0,
            filtered_empty_tags: 0,
        }
    }

    /// Wall-clock time since the run started.
    pub fn elapsed(&self) -> Duration {
        self.completed_in
            .unwrap_or_else(|| self.started_at.elapsed())
    }

    /// Freeze the elapsed duration at the end of a run.
    pub fn finish(&mut self) {
        if self.completed_in.is_none() {
            self.completed_in = Some(self.started_at.elapsed());
        }
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

    #[test]
    fn finished_metrics_keep_stable_elapsed_time() {
        let mut metrics = RunMetrics::start();
        metrics.finish();
        let elapsed = metrics.elapsed();

        std::thread::sleep(Duration::from_millis(5));

        assert_eq!(metrics.elapsed(), elapsed);
    }
}
