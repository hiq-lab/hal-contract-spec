# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-07-02

Reference implementations synced to spec v2.1–v2.3 (they previously
implemented v2.0 while the spec document moved ahead).

### Added
- `JobStatus::ResultExpired` terminal state (spec v2.1 §5.1) — Rust, Python, Haskell
- `HalError::ResultExpired` error variant (spec v2.1 §7; 14 variants total)
- `Capabilities.max_circuit_ops: Option<u32>` (spec v2.1 §4.1)
- `GateSet::quantinuum()` / `GateSet::aqt()` and matching `Capabilities`
  factories (spec v2.1 §8)
- `submit_with_parameters()` provided trait method for parametric circuits
  (spec v2.3 §3.3 rule 8) — default delegates to `submit()` when empty,
  returns `Unsupported` otherwise

### Changed
- **Breaking:** `validate(circuit)` → `validate(circuit, shots)` (spec v2.1
  §3.2); validate is the authoritative pre-flight check per §3.3 rule 3
- **Breaking (Rust):** `BackendAvailability.estimated_wait_secs: Option<f64>`
  → `estimated_wait: Option<std::time::Duration>` per §4.5
- `wait()` default implementation handles `ResultExpired`
- Mock example (`examples/rust-mock/`) routes `submit()` through `validate()`
  per §3.3 rule 4

## [0.1.0] - 2026-02-20

### Added
- Initial extraction from arvak-hal as standalone specification
- `Backend<C>` trait — generic over circuit type, zero Arvak dependencies
- `Capabilities`, `GateSet`, `Topology`, `NoiseProfile` types
- `JobId`, `JobStatus` with state machine invariants
- `Counts`, `ExecutionResult` for measurement results
- `HalError` with 13 categorized error variants
- Reference gate sets for IQM, IBM Eagle/Heron, Rigetti, IonQ, neutral atom
- Formal specification document (`spec/v2.md`)
- Rust reference crate (`rust/`)
- Python reference implementation (`python/`)
- Haskell reference implementation (`haskell/`)
- Mock backend example (`examples/rust-mock/`)
