//! Backend trait and availability types.
//!
//! # HAL Contract v2
//!
//! The [`Backend`] trait defines the lifecycle for interacting with a
//! quantum backend:
//!
//! ```text
//!   capabilities() ──→ validate() ──→ submit() ──→ status() ──→ result()
//!    (sync, &ref)       (async)       (async)      (async)      (async)
//! ```
//!
//! ## Design principles
//!
//! - **Async-native**: all I/O methods are async.
//! - **Thread-safe**: `Send + Sync` bound enables shared ownership.
//! - **Minimal**: only the methods needed for the job lifecycle.
//! - **Infallible introspection**: `capabilities()` is synchronous and
//!   infallible — a backend that cannot report capabilities without I/O
//!   is not correctly initialized.
//! - **Circuit-generic**: the trait is parameterized over `C`, the circuit
//!   type, so it is independent of any specific IR.
//!
//! ## Method table
//!
//! | Method | Kind | Required | Returns |
//! |--------|------|----------|---------|
//! | `name()` | sync | yes | `&str` |
//! | `capabilities()` | sync | yes | `&Capabilities` |
//! | `availability()` | async | yes | `HalResult<BackendAvailability>` |
//! | `validate()` | async | yes | `HalResult<ValidationResult>` |
//! | `submit()` | async | yes | `HalResult<JobId>` |
//! | `status()` | async | yes | `HalResult<JobStatus>` |
//! | `result()` | async | yes | `HalResult<ExecutionResult>` |
//! | `cancel()` | async | yes | `HalResult<()>` |
//! | `wait()` | async | provided | `HalResult<ExecutionResult>` |

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::capability::Capabilities;
use crate::error::HalResult;
use crate::job::{JobId, JobStatus};
use crate::result::ExecutionResult;

/// Trait for quantum backends.
///
/// This trait defines the interface that all quantum backends MUST implement.
/// It covers the full job lifecycle: introspection, validation, submission,
/// status polling, result retrieval, and cancellation.
///
/// The trait is generic over `C`, the circuit type — making it independent
/// of any specific intermediate representation.
///
/// # Contract
///
/// - `capabilities()` MUST be synchronous and infallible. Capabilities
///   MUST be cached at construction time.
/// - `availability()` SHOULD perform a lightweight liveness check.
/// - `validate()` MUST check the circuit against backend constraints
///   before submission.
/// - `submit()` MUST return `JobId` with initial status `Queued`.
/// - `result()` MUST only be called when status is `Completed`.
/// - `wait()` has a default implementation (500ms poll, 5-minute timeout).
#[async_trait]
pub trait Backend<C>: Send + Sync {
    /// Get the name of this backend.
    fn name(&self) -> &str;

    /// Get the capabilities of this backend.
    ///
    /// This method is synchronous and infallible. Implementations MUST
    /// cache capabilities at construction time and return a reference.
    fn capabilities(&self) -> &Capabilities;

    /// Check backend availability with queue depth information.
    ///
    /// Returns richer information than a simple boolean: queue depth,
    /// estimated wait time, and an optional status message. This enables
    /// intelligent routing decisions by schedulers.
    async fn availability(&self) -> HalResult<BackendAvailability>;

    /// Validate a circuit against backend constraints.
    ///
    /// SHOULD check at minimum:
    /// - Qubit count vs `capabilities().num_qubits`
    /// - Gate support vs `capabilities().gate_set`
    ///
    /// Returns a three-state result: `Valid`, `Invalid`, or
    /// `RequiresTranspilation`. The third state lets an orchestrator
    /// decide to compile and retry vs. route elsewhere.
    async fn validate(&self, circuit: &C) -> HalResult<ValidationResult>;

    /// Submit a circuit for execution.
    ///
    /// Returns a job ID that can be used to check status and retrieve results.
    /// The job MUST start in `Queued` status.
    async fn submit(&self, circuit: &C, shots: u32) -> HalResult<JobId>;

    /// Get the status of a job.
    async fn status(&self, job_id: &JobId) -> HalResult<JobStatus>;

    /// Get the result of a completed job.
    ///
    /// MUST only be called when `status()` returns `Completed`.
    async fn result(&self, job_id: &JobId) -> HalResult<ExecutionResult>;

    /// Cancel a running job.
    async fn cancel(&self, job_id: &JobId) -> HalResult<()>;

    /// Wait for a job to complete and return its result.
    ///
    /// Default implementation polls every 500ms for up to 5 minutes.
    async fn wait(&self, job_id: &JobId) -> HalResult<ExecutionResult> {
        use crate::error::HalError;
        use tokio::time::sleep;

        let poll_interval = Duration::from_millis(500);
        let max_polls = 600; // 5 minutes max

        for _ in 0..max_polls {
            let status = self.status(job_id).await?;

            match status {
                JobStatus::Completed => return self.result(job_id).await,
                JobStatus::Failed(msg) => return Err(HalError::JobFailed(msg)),
                JobStatus::Cancelled => return Err(HalError::JobCancelled),
                JobStatus::Queued | JobStatus::Running => {
                    sleep(poll_interval).await;
                }
            }
        }

        Err(HalError::Timeout(job_id.0.clone()))
    }
}

/// Backend availability information.
///
/// Provides richer availability data than a simple boolean, enabling
/// schedulers to make informed routing decisions based on queue depth
/// and estimated wait times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendAvailability {
    /// Whether the backend is currently accepting jobs.
    pub is_available: bool,
    /// Number of jobs currently in queue (if known).
    pub queue_depth: Option<u32>,
    /// Estimated wait time for a new job in seconds (if known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_wait_secs: Option<f64>,
    /// Human-readable status message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
}

impl BackendAvailability {
    /// Create availability for a backend that is always available.
    ///
    /// Typical for simulators — zero queue, zero wait.
    pub fn always_available() -> Self {
        Self {
            is_available: true,
            queue_depth: Some(0),
            estimated_wait_secs: Some(0.0),
            status_message: None,
        }
    }

    /// Create availability for an offline backend.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            is_available: false,
            queue_depth: None,
            estimated_wait_secs: None,
            status_message: Some(reason.into()),
        }
    }
}

/// Result of circuit validation against backend constraints.
///
/// The three-state return is deliberate:
/// - `Valid` — the circuit can be submitted as-is.
/// - `Invalid` — the circuit cannot run on this backend.
/// - `RequiresTranspilation` — the circuit needs compilation but
///   could run after transpilation.
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Circuit is valid and can be submitted directly.
    Valid,
    /// Circuit is invalid for this backend.
    Invalid {
        /// Reasons the circuit is invalid.
        reasons: Vec<String>,
    },
    /// Circuit could run after transpilation.
    RequiresTranspilation {
        /// What transpilation is needed.
        details: String,
    },
}

impl ValidationResult {
    /// Check if the circuit is valid (can be submitted as-is).
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_availability_always_available() {
        let avail = BackendAvailability::always_available();
        assert!(avail.is_available);
        assert_eq!(avail.queue_depth, Some(0));
        assert!(avail.status_message.is_none());
    }

    #[test]
    fn test_backend_availability_unavailable() {
        let avail = BackendAvailability::unavailable("maintenance");
        assert!(!avail.is_available);
        assert_eq!(avail.status_message, Some("maintenance".to_string()));
    }

    #[test]
    fn test_validation_result_is_valid() {
        assert!(ValidationResult::Valid.is_valid());
        assert!(!ValidationResult::Invalid { reasons: vec![] }.is_valid());
        assert!(
            !ValidationResult::RequiresTranspilation {
                details: String::new()
            }
            .is_valid()
        );
    }
}
