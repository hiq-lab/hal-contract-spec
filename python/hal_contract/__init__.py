"""HAL Contract — Orchestration Interface Specification for Quantum-HPC Workflows.

Python reference implementation of the HAL Contract v2 specification.
"""

from hal_contract.backend import (
    Backend,
    BackendAvailability,
    Capabilities,
    Counts,
    ExecutionResult,
    GateSet,
    HalError,
    JobId,
    JobStatus,
    NoiseProfile,
    Topology,
    TopologyKind,
    ValidationResult,
)

__all__ = [
    "Backend",
    "BackendAvailability",
    "Capabilities",
    "Counts",
    "ExecutionResult",
    "GateSet",
    "HalError",
    "JobId",
    "JobStatus",
    "NoiseProfile",
    "Topology",
    "TopologyKind",
    "ValidationResult",
]

__version__ = "0.1.0"
