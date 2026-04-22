//! Permission Check latency/result reporting.
//!
//! The engine has no `metrics`/`prometheus` dependency — the domain crate
//! stays storage- and observability-agnostic. Instead it accepts
//! [`PermissionCheckMetrics`] from the caller and records one sample per
//! invocation of [`crate::permissions::check`].
//!
//! - Tests pass [`NoopMetrics`].
//! - P6's server code will implement this trait over a
//!   `prometheus::HistogramVec` with labels `{result, failed_step}`, which
//!   is the histogram named `phi_permission_check_duration_seconds`
//!   (commitment C10 in the M1 plan).

use std::time::Duration;

/// Caller-provided metrics sink. Implementations should be cheap; the
/// engine calls `record` exactly once per check.
pub trait PermissionCheckMetrics: Send + Sync {
    /// `duration` is wall time of the pipeline. `result_label` is one of
    /// `"allowed"` / `"denied"` / `"pending"` (see [`crate::permissions::
    /// decision::Decision::metric_result_label`]). `failed_step_label` is
    /// [`Some("0" | "1" | "2" | "2a" | "3" | "4" | "5" | "6")`] on denials
    /// and `None` otherwise.
    fn record(&self, duration: Duration, result_label: &str, failed_step_label: Option<&str>);
}

/// A [`PermissionCheckMetrics`] that drops every sample. Used by tests and
/// by domain proptests.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetrics;

impl PermissionCheckMetrics for NoopMetrics {
    fn record(&self, _duration: Duration, _result_label: &str, _failed_step_label: Option<&str>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// A capturing sink used by the engine tests.
    #[derive(Default)]
    struct CapturingMetrics(Mutex<Vec<(Duration, String, Option<String>)>>);

    impl PermissionCheckMetrics for CapturingMetrics {
        fn record(&self, d: Duration, r: &str, f: Option<&str>) {
            self.0
                .lock()
                .unwrap()
                .push((d, r.to_string(), f.map(|s| s.to_string())));
        }
    }

    #[test]
    fn noop_compiles_and_drops_samples() {
        let m = NoopMetrics;
        m.record(Duration::from_millis(1), "allowed", None);
        m.record(Duration::from_millis(2), "denied", Some("3"));
    }

    #[test]
    fn capturing_sink_collects_samples() {
        let sink = Arc::new(CapturingMetrics::default());
        sink.record(Duration::from_millis(5), "allowed", None);
        sink.record(Duration::from_millis(6), "denied", Some("0"));
        let captured = sink.0.lock().unwrap().clone();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0].1, "allowed");
        assert_eq!(captured[1].2.as_deref(), Some("0"));
    }
}
