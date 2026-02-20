# Changelog

All notable changes to this project will be documented in this file.

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
