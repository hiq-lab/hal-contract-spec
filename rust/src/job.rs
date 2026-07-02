//! Job lifecycle types.
//!
//! # HAL Contract v2
//!
//! The job state machine:
//!
//! ```text
//!   submit() ──→ Queued ──→ Running ──→ Completed ──→ ResultExpired
//!                  │           │
//!                  │           ├──→ Failed(reason)
//!                  │           │
//!                  └───────────┴──→ Cancelled
//! ```
//!
//! **Invariants:**
//! - A job created by `submit()` starts in `Queued`. Synchronous backends
//!   MAY complete before the first `status()` call — `Queued` is the initial
//!   recorded state, not a guaranteed observation.
//! - Transitions are monotonic — a job never moves backward or re-enters
//!   execution.
//! - `Failed`, `Cancelled`, and `ResultExpired` are permanent.
//! - `Completed` is terminal for execution, but MAY transition to
//!   `ResultExpired` if the backend purges results after a retention window.
//! - `result()` is only valid when status is `Completed`.

use serde::{Deserialize, Serialize};

/// Unique identifier for a job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

impl JobId {
    /// Create a new job ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for JobId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for JobId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Status of a job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is waiting in queue.
    Queued,
    /// Job is currently running.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed with an error message.
    Failed(String),
    /// Job was cancelled.
    Cancelled,
    /// Job completed but results are no longer available (e.g. the backend
    /// has a retention window). Only reachable from `Completed`; distinct
    /// from `Failed` — the job ran successfully. `result()` MUST return
    /// `HalError::ResultExpired` in this state.
    ResultExpired,
}

impl JobStatus {
    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed
                | JobStatus::Failed(_)
                | JobStatus::Cancelled
                | JobStatus::ResultExpired
        )
    }

    /// Check if the job is still pending (queued or running).
    pub fn is_pending(&self) -> bool {
        matches!(self, JobStatus::Queued | JobStatus::Running)
    }

    /// Check if the job completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, JobStatus::Completed)
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "Queued"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed(msg) => write!(f, "Failed: {msg}"),
            JobStatus::Cancelled => write!(f, "Cancelled"),
            JobStatus::ResultExpired => write!(f, "ResultExpired"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_terminal() {
        assert!(!JobStatus::Queued.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Failed("error".into()).is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(JobStatus::ResultExpired.is_terminal());
    }

    #[test]
    fn test_result_expired_is_not_success_or_pending() {
        assert!(!JobStatus::ResultExpired.is_success());
        assert!(!JobStatus::ResultExpired.is_pending());
        assert_eq!(JobStatus::ResultExpired.to_string(), "ResultExpired");
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Queued.to_string(), "Queued");
        assert_eq!(JobStatus::Running.to_string(), "Running");
        assert_eq!(
            JobStatus::Failed("timeout".into()).to_string(),
            "Failed: timeout"
        );
    }

    #[test]
    fn test_job_id_from() {
        let id: JobId = "job-123".into();
        assert_eq!(id.0, "job-123");
        assert_eq!(id.to_string(), "job-123");
    }
}
