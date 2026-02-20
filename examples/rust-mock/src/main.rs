//! Minimal mock backend implementing the HAL Contract v2.
//!
//! This example demonstrates how to implement the `Backend` trait
//! for a simple in-memory simulator.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use hal_contract::{
    Backend, BackendAvailability, Capabilities, Counts, ExecutionResult, HalError, HalResult,
    JobId, JobStatus, ValidationResult,
};

/// A simple circuit type for demonstration.
struct SimpleCircuit {
    num_qubits: u32,
    gates: Vec<String>,
}

/// In-memory mock backend.
struct MockBackend {
    capabilities: Capabilities,
    jobs: Mutex<HashMap<String, (JobStatus, Option<ExecutionResult>)>>,
    next_id: Mutex<u64>,
}

impl MockBackend {
    fn new(num_qubits: u32) -> Self {
        Self {
            capabilities: Capabilities::simulator(num_qubits),
            jobs: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Backend<SimpleCircuit> for MockBackend {
    fn name(&self) -> &str {
        "mock-simulator"
    }

    fn capabilities(&self) -> &Capabilities {
        &self.capabilities
    }

    async fn availability(&self) -> HalResult<BackendAvailability> {
        Ok(BackendAvailability::always_available())
    }

    async fn validate(&self, circuit: &SimpleCircuit) -> HalResult<ValidationResult> {
        if circuit.num_qubits > self.capabilities.num_qubits {
            return Ok(ValidationResult::Invalid {
                reasons: vec![format!(
                    "Circuit requires {} qubits, backend has {}",
                    circuit.num_qubits, self.capabilities.num_qubits
                )],
            });
        }

        for gate in &circuit.gates {
            if !self.capabilities.gate_set.contains(gate) {
                return Ok(ValidationResult::Invalid {
                    reasons: vec![format!("Unsupported gate: {gate}")],
                });
            }
        }

        Ok(ValidationResult::Valid)
    }

    async fn submit(&self, circuit: &SimpleCircuit, shots: u32) -> HalResult<JobId> {
        if shots == 0 || shots > self.capabilities.max_shots {
            return Err(HalError::InvalidShots(format!(
                "shots must be 1..={}",
                self.capabilities.max_shots
            )));
        }

        let id = {
            let mut next = self.next_id.lock().unwrap();
            *next += 1;
            format!("mock-{}", *next)
        };

        // Simulate execution: produce random-ish counts
        let mut counts = Counts::new();
        let all_zeros = "0".repeat(circuit.num_qubits as usize);
        let all_ones = "1".repeat(circuit.num_qubits as usize);
        counts.insert(&all_zeros, (shots / 2).into());
        counts.insert(&all_ones, (shots - shots / 2).into());

        let result = ExecutionResult::new(counts, shots).with_execution_time(42);

        self.jobs
            .lock()
            .unwrap()
            .insert(id.clone(), (JobStatus::Completed, Some(result)));

        Ok(JobId::new(id))
    }

    async fn status(&self, job_id: &JobId) -> HalResult<JobStatus> {
        self.jobs
            .lock()
            .unwrap()
            .get(&job_id.0)
            .map(|(s, _)| s.clone())
            .ok_or_else(|| HalError::JobNotFound(job_id.0.clone()))
    }

    async fn result(&self, job_id: &JobId) -> HalResult<ExecutionResult> {
        self.jobs
            .lock()
            .unwrap()
            .get(&job_id.0)
            .and_then(|(_, r)| r.clone())
            .ok_or_else(|| HalError::JobNotFound(job_id.0.clone()))
    }

    async fn cancel(&self, job_id: &JobId) -> HalResult<()> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some((status, _)) = jobs.get_mut(&job_id.0) {
            if !status.is_terminal() {
                *status = JobStatus::Cancelled;
            }
            Ok(())
        } else {
            Err(HalError::JobNotFound(job_id.0.clone()))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = MockBackend::new(4);

    println!("Backend: {}", backend.name());
    println!("Qubits:  {}", backend.capabilities().num_qubits);
    println!();

    // Check availability
    let avail = backend.availability().await?;
    println!("Available: {} (queue: {:?})", avail.is_available, avail.queue_depth);

    // Create a simple circuit
    let circuit = SimpleCircuit {
        num_qubits: 2,
        gates: vec!["h".into(), "cx".into()],
    };

    // Validate
    let validation = backend.validate(&circuit).await?;
    println!("Valid: {}", validation.is_valid());

    // Submit
    let job_id = backend.submit(&circuit, 1000).await?;
    println!("Job ID: {job_id}");

    // Get result (mock completes instantly)
    let result = backend.result(&job_id).await?;
    println!("Shots:  {}", result.shots);
    println!("Time:   {}ms", result.execution_time_ms.unwrap_or(0));
    println!();

    // Print counts
    println!("Results:");
    for (bitstring, count) in result.counts.sorted() {
        println!("  {bitstring}: {count}");
    }

    if let Some((bitstring, prob)) = result.most_frequent() {
        println!("\nMost frequent: {bitstring} ({:.1}%)", prob * 100.0);
    }

    Ok(())
}
