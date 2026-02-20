"""HAL Contract v2 — Python reference implementation.

This module provides abstract base classes and data types matching
the HAL Contract v2 specification. Any quantum backend can implement
the Backend ABC to participate in orchestrated workflows.

The Backend class is generic over a circuit type C, making it
independent of any specific intermediate representation.
"""

from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import Generic, TypeVar

C = TypeVar("C")
"""Type variable for the circuit type."""


# ── Error types ──────────────────────────────────────────────────────


class HalError(Exception):
    """Base error for HAL operations."""


class BackendUnavailableError(HalError):
    """Backend is not available (transient — retry with backoff)."""


class TimeoutError(HalError):
    """Timeout waiting for job (transient — retry with backoff)."""


class InvalidCircuitError(HalError):
    """Invalid circuit (permanent — fix input)."""


class CircuitTooLargeError(HalError):
    """Circuit exceeds backend capabilities (permanent — fix input)."""


class InvalidShotsError(HalError):
    """Invalid number of shots (permanent — fix input)."""


class UnsupportedError(HalError):
    """Unsupported feature (permanent)."""


class SubmissionFailedError(HalError):
    """Job submission failed."""


class JobFailedError(HalError):
    """Job execution failed (terminal)."""


class JobCancelledError(HalError):
    """Job was cancelled (terminal)."""


class JobNotFoundError(HalError):
    """Job not found."""


class AuthenticationFailedError(HalError):
    """Authentication failed."""


class ConfigurationError(HalError):
    """Configuration error."""


class BackendError(HalError):
    """Generic backend error."""


# ── Data types ───────────────────────────────────────────────────────


class JobId(str):
    """Unique identifier for a job.

    Opaque string — backends generate these, consumers treat them as opaque.
    """


class JobStatus(Enum):
    """Status of a job.

    State machine::

        submit() ──→ QUEUED ──→ RUNNING ──→ COMPLETED
                       │           │
                       │           ├──→ FAILED
                       │           │
                       └───────────┴──→ CANCELLED
    """

    QUEUED = auto()
    RUNNING = auto()
    COMPLETED = auto()
    FAILED = auto()
    CANCELLED = auto()

    @property
    def is_terminal(self) -> bool:
        return self in (JobStatus.COMPLETED, JobStatus.FAILED, JobStatus.CANCELLED)

    @property
    def is_pending(self) -> bool:
        return self in (JobStatus.QUEUED, JobStatus.RUNNING)

    @property
    def is_success(self) -> bool:
        return self is JobStatus.COMPLETED


class TopologyKind(Enum):
    """Kind of qubit topology."""

    FULLY_CONNECTED = auto()
    LINEAR = auto()
    STAR = auto()
    GRID = auto()
    HEAVY_HEX = auto()
    CUSTOM = auto()
    NEUTRAL_ATOM = auto()


@dataclass(frozen=True)
class NoiseProfile:
    """Device-wide noise averages.

    Fidelity values in [0.0, 1.0] (1.0 = perfect).
    Time values in microseconds.
    """

    t1: float | None = None
    t2: float | None = None
    single_qubit_fidelity: float | None = None
    two_qubit_fidelity: float | None = None
    readout_fidelity: float | None = None
    gate_time: float | None = None


@dataclass
class GateSet:
    """Gate set supported by a backend.

    Gate names follow OpenQASM 3 convention (lowercase).
    If ``native`` is empty, all supported gates are considered native.
    """

    single_qubit: list[str] = field(default_factory=list)
    two_qubit: list[str] = field(default_factory=list)
    three_qubit: list[str] = field(default_factory=list)
    native: list[str] = field(default_factory=list)

    def contains(self, gate: str) -> bool:
        return gate in self.single_qubit or gate in self.two_qubit or gate in self.three_qubit

    def is_native(self, gate: str) -> bool:
        if not self.native:
            return self.contains(gate)
        return gate in self.native

    @staticmethod
    def iqm() -> GateSet:
        return GateSet(
            single_qubit=["prx"],
            two_qubit=["cz"],
            native=["prx", "cz"],
        )

    @staticmethod
    def ibm_eagle() -> GateSet:
        return GateSet(
            single_qubit=["rz", "sx", "x", "id"],
            two_qubit=["ecr"],
            native=["rz", "sx", "x", "ecr"],
        )

    @staticmethod
    def ibm_heron() -> GateSet:
        return GateSet(
            single_qubit=["rz", "sx", "x", "id", "rx", "h"],
            two_qubit=["cz", "rzz"],
            native=["rz", "sx", "x", "cz", "id", "rx", "h", "rzz"],
        )

    @staticmethod
    def universal() -> GateSet:
        return GateSet(
            single_qubit=[
                "id", "x", "y", "z", "h", "s", "sdg", "t", "tdg",
                "sx", "sxdg", "rx", "ry", "rz", "p", "u", "prx",
            ],
            two_qubit=[
                "cx", "cy", "cz", "ch", "swap", "iswap",
                "crx", "cry", "crz", "cp", "rxx", "ryy", "rzz",
            ],
            three_qubit=["ccx", "cswap"],
        )

    @staticmethod
    def rigetti() -> GateSet:
        return GateSet(
            single_qubit=["rx", "rz"],
            two_qubit=["cz"],
            native=["rx", "rz", "cz"],
        )

    @staticmethod
    def ionq() -> GateSet:
        return GateSet(
            single_qubit=["rx", "ry", "rz"],
            two_qubit=["xx"],
            native=["rx", "ry", "rz", "xx"],
        )

    @staticmethod
    def neutral_atom() -> GateSet:
        return GateSet(
            single_qubit=["rz", "rx", "ry"],
            two_qubit=["cz"],
            native=["rz", "rx", "ry", "cz"],
        )


@dataclass
class Topology:
    """Qubit connectivity topology.

    All edges are bidirectional.
    """

    kind: TopologyKind
    edges: list[tuple[int, int]] = field(default_factory=list)

    def is_connected(self, q1: int, q2: int) -> bool:
        return any((a == q1 and b == q2) or (a == q2 and b == q1) for a, b in self.edges)

    @staticmethod
    def linear(n: int) -> Topology:
        return Topology(kind=TopologyKind.LINEAR, edges=[(i, i + 1) for i in range(n - 1)])

    @staticmethod
    def star(n: int) -> Topology:
        return Topology(kind=TopologyKind.STAR, edges=[(0, i) for i in range(1, n)])

    @staticmethod
    def full(n: int) -> Topology:
        return Topology(
            kind=TopologyKind.FULLY_CONNECTED,
            edges=[(i, j) for i in range(n) for j in range(i + 1, n)],
        )

    @staticmethod
    def grid(rows: int, cols: int) -> Topology:
        edges = []
        for r in range(rows):
            for c in range(cols):
                idx = r * cols + c
                if c + 1 < cols:
                    edges.append((idx, idx + 1))
                if r + 1 < rows:
                    edges.append((idx, idx + cols))
        return Topology(kind=TopologyKind.GRID, edges=edges)

    @staticmethod
    def custom(edges: list[tuple[int, int]]) -> Topology:
        return Topology(kind=TopologyKind.CUSTOM, edges=edges)


@dataclass
class Capabilities:
    """Hardware capabilities of a quantum backend."""

    name: str
    num_qubits: int
    gate_set: GateSet
    topology: Topology
    max_shots: int
    is_simulator: bool
    features: list[str] = field(default_factory=list)
    noise_profile: NoiseProfile | None = None

    @staticmethod
    def simulator(num_qubits: int) -> Capabilities:
        return Capabilities(
            name="simulator",
            num_qubits=num_qubits,
            gate_set=GateSet.universal(),
            topology=Topology.full(num_qubits),
            max_shots=100_000,
            is_simulator=True,
            features=["statevector", "unitary"],
        )

    @staticmethod
    def iqm(name: str, num_qubits: int) -> Capabilities:
        return Capabilities(
            name=name,
            num_qubits=num_qubits,
            gate_set=GateSet.iqm(),
            topology=Topology.star(num_qubits),
            max_shots=20_000,
            is_simulator=False,
        )


@dataclass
class BackendAvailability:
    """Backend availability information."""

    is_available: bool
    queue_depth: int | None = None
    estimated_wait_secs: float | None = None
    status_message: str | None = None

    @staticmethod
    def always_available() -> BackendAvailability:
        return BackendAvailability(is_available=True, queue_depth=0, estimated_wait_secs=0.0)

    @staticmethod
    def unavailable(reason: str) -> BackendAvailability:
        return BackendAvailability(is_available=False, status_message=reason)


@dataclass
class ValidationResult:
    """Result of circuit validation."""

    is_valid: bool
    reasons: list[str] = field(default_factory=list)
    requires_transpilation: bool = False
    transpilation_details: str | None = None

    @staticmethod
    def valid() -> ValidationResult:
        return ValidationResult(is_valid=True)

    @staticmethod
    def invalid(reasons: list[str]) -> ValidationResult:
        return ValidationResult(is_valid=False, reasons=reasons)

    @staticmethod
    def needs_transpilation(details: str) -> ValidationResult:
        return ValidationResult(is_valid=False, requires_transpilation=True, transpilation_details=details)


@dataclass
class Counts:
    """Measurement counts from circuit execution.

    Bitstring ordering follows OpenQASM 3 convention:
    rightmost bit = lowest qubit index.
    """

    _counts: dict[str, int] = field(default_factory=dict)

    def insert(self, bitstring: str, count: int) -> None:
        self._counts[bitstring] = self._counts.get(bitstring, 0) + count

    def get(self, bitstring: str) -> int:
        return self._counts.get(bitstring, 0)

    def total_shots(self) -> int:
        return sum(self._counts.values())

    def most_frequent(self) -> tuple[str, int] | None:
        if not self._counts:
            return None
        return max(self._counts.items(), key=lambda x: x[1])

    def probabilities(self) -> dict[str, float]:
        total = self.total_shots()
        if total == 0:
            return {}
        return {k: v / total for k, v in self._counts.items()}

    def items(self):
        return self._counts.items()

    def __len__(self) -> int:
        return len(self._counts)

    def __bool__(self) -> bool:
        return bool(self._counts)

    @staticmethod
    def from_dict(d: dict[str, int]) -> Counts:
        c = Counts()
        c._counts = dict(d)
        return c


@dataclass
class ExecutionResult:
    """Result of circuit execution."""

    counts: Counts
    shots: int
    execution_time_ms: int | None = None
    metadata: dict | None = None

    def probabilities(self) -> dict[str, float]:
        return self.counts.probabilities()

    def most_frequent(self) -> tuple[str, float] | None:
        total = self.counts.total_shots()
        if total == 0:
            return None
        result = self.counts.most_frequent()
        if result is None:
            return None
        bitstring, count = result
        return bitstring, count / total


# ── Backend ABC ──────────────────────────────────────────────────────


class Backend(ABC, Generic[C]):
    """Abstract base class for quantum backends.

    Implements the HAL Contract v2 specification. All quantum backends
    MUST subclass this and implement the abstract methods.

    The class is generic over C, the circuit type.

    Lifecycle::

        capabilities() ──→ validate() ──→ submit() ──→ status() ──→ result()
         (sync, ref)        (async)       (async)      (async)      (async)
    """

    @abstractmethod
    def name(self) -> str:
        """Get the name of this backend."""
        ...

    @abstractmethod
    def capabilities(self) -> Capabilities:
        """Get the capabilities of this backend.

        MUST be synchronous and return cached capabilities.
        """
        ...

    @abstractmethod
    async def availability(self) -> BackendAvailability:
        """Check backend availability with queue depth information."""
        ...

    @abstractmethod
    async def validate(self, circuit: C) -> ValidationResult:
        """Validate a circuit against backend constraints."""
        ...

    @abstractmethod
    async def submit(self, circuit: C, shots: int) -> JobId:
        """Submit a circuit for execution. Job MUST start in QUEUED status."""
        ...

    @abstractmethod
    async def status(self, job_id: JobId) -> JobStatus:
        """Get the status of a job."""
        ...

    @abstractmethod
    async def result(self, job_id: JobId) -> ExecutionResult:
        """Get the result of a completed job. Only valid when status is COMPLETED."""
        ...

    @abstractmethod
    async def cancel(self, job_id: JobId) -> None:
        """Cancel a running job."""
        ...

    async def wait(self, job_id: JobId) -> ExecutionResult:
        """Wait for a job to complete and return its result.

        Default implementation polls every 500ms for up to 5 minutes.
        """
        for _ in range(600):
            s = await self.status(job_id)
            if s is JobStatus.COMPLETED:
                return await self.result(job_id)
            if s is JobStatus.FAILED:
                raise JobFailedError(f"Job {job_id} failed")
            if s is JobStatus.CANCELLED:
                raise JobCancelledError(f"Job {job_id} cancelled")
            await asyncio.sleep(0.5)
        raise TimeoutError(f"Timeout waiting for job {job_id}")
