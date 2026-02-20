//! HAL Contract — Orchestration Interface Specification for Quantum-HPC Workflows
//!
//! This crate provides the **HAL Contract v2** specification types: a vendor-neutral
//! interface for interacting with quantum backends. Any backend — simulator, cloud
//! QPU, or HPC cluster — implements the [`Backend`] trait to participate in
//! orchestrated quantum workflows.
//!
//! # Overview
//!
//! The HAL Contract defines:
//! - A [`Backend`] trait covering the full job lifecycle
//! - [`Capabilities`] to describe hardware features and constraints
//! - [`GateSet`], [`Topology`], [`NoiseProfile`] for hardware introspection
//! - [`JobId`] / [`JobStatus`] for job tracking
//! - [`ExecutionResult`] / [`Counts`] for measurement results
//! - [`HalError`] with 13 categorized error variants
//!
//! # The Backend Trait
//!
//! The trait is generic over a circuit type `C`, making it independent of
//! any specific IR:
//!
//! ```ignore
//! use hal_contract::{Backend, Capabilities, HalResult, BackendAvailability, ValidationResult, JobId, JobStatus, ExecutionResult};
//! use async_trait::async_trait;
//!
//! struct MyBackend { /* ... */ }
//!
//! #[async_trait]
//! impl Backend<MyCircuit> for MyBackend {
//!     fn name(&self) -> &str { "my_backend" }
//!     fn capabilities(&self) -> &Capabilities { /* ... */ }
//!     // ... implement remaining methods
//! }
//! ```
//!
//! # Lifecycle
//!
//! ```text
//!   capabilities() ──→ validate() ──→ submit() ──→ status() ──→ result()
//!    (sync, &ref)       (async)       (async)      (async)      (async)
//! ```
//!
//! See the [spec](https://github.com/hiq-lab/hal-contract-spec/blob/main/spec/v2.md)
//! for the full formal specification.

pub mod backend;
pub mod capability;
pub mod error;
pub mod job;
pub mod result;

pub use backend::{Backend, BackendAvailability, ValidationResult};
pub use capability::{Capabilities, GateSet, NoiseProfile, Topology, TopologyKind};
pub use error::{HalError, HalResult};
pub use job::{JobId, JobStatus};
pub use result::{Counts, ExecutionResult};
