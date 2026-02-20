# HAL Contract

**Orchestration Interface Specification for Quantum-HPC Workflows**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Spec](https://img.shields.io/badge/spec-v2-green.svg)](spec/v2.md)

The HAL Contract defines a vendor-neutral interface for quantum backends. Any backend — simulator, cloud QPU, or HPC cluster — implements the `Backend` trait to participate in orchestrated quantum workflows.

**Website:** [hal-contract.org](https://hal-contract.org)
**Reference implementation:** [Arvak](https://arvak.io)

## Overview

The contract covers the full job lifecycle:

```text
  capabilities() ──→ validate() ──→ submit() ──→ status() ──→ result()
   (sync, cached)     (async)       (async)      (async)      (async)
```

| Method | Kind | Returns |
|--------|------|---------|
| `name()` | sync | Backend identifier |
| `capabilities()` | sync | Hardware descriptor |
| `availability()` | async | Queue depth, wait time |
| `validate()` | async | Valid / Invalid / RequiresTranspilation |
| `submit()` | async | Job ID |
| `status()` | async | Queued / Running / Completed / Failed / Cancelled |
| `result()` | async | Measurement counts |
| `cancel()` | async | Best-effort cancellation |

## Reference Implementations

### Rust

```toml
[dependencies]
hal-contract = "0.1"
```

```rust
use hal_contract::{Backend, Capabilities, HalResult, JobId, JobStatus, ExecutionResult};
use async_trait::async_trait;

struct MyBackend {
    capabilities: Capabilities,
}

#[async_trait]
impl Backend<MyCircuit> for MyBackend {
    fn name(&self) -> &str { "my-backend" }
    fn capabilities(&self) -> &Capabilities { &self.capabilities }
    // ... implement remaining methods
}
```

See [`examples/rust-mock/`](examples/rust-mock/) for a complete working example.

### Python

```python
from hal_contract import Backend, Capabilities, JobId, JobStatus

class MyBackend(Backend[MyCircuit]):
    def name(self) -> str:
        return "my-backend"

    def capabilities(self) -> Capabilities:
        return self._capabilities

    async def validate(self, circuit: MyCircuit) -> ValidationResult:
        ...

    async def submit(self, circuit: MyCircuit, shots: int) -> JobId:
        ...

    # ... implement remaining methods
```

### Haskell

```haskell
import HAL.Contract.Backend

data MyBackend = MyBackend { mbCapabilities :: Capabilities }

instance Backend MyBackend where
  type Circuit MyBackend = MyCircuit
  backendName = const "my-backend"
  capabilities = mbCapabilities
  -- ... implement remaining methods
```

## Specification

The full formal specification is at [`spec/v2.md`](spec/v2.md).

Key design principles:
- **Async-native** — All I/O methods are async
- **Thread-safe** — `Send + Sync` enables shared ownership
- **Minimal** — Only the job lifecycle, nothing more
- **Circuit-generic** — Parameterized over circuit type, no IR dependency

## Project Structure

```
hal-contract/
├── spec/v2.md              # Formal specification
├── rust/                   # Rust reference crate
├── python/                 # Python reference (ABC + dataclasses)
├── haskell/                # Haskell reference (type class)
└── examples/rust-mock/     # Minimal mock backend example
```

## Reference Gate Sets

Built-in presets for common quantum hardware:

| Vendor | Factory | Native Gates |
|--------|---------|-------------|
| IQM | `GateSet::iqm()` | prx, cz |
| IBM Eagle | `GateSet::ibm_eagle()` | rz, sx, x, ecr |
| IBM Heron | `GateSet::ibm_heron()` | rz, sx, x, cz, rx, h, rzz |
| Rigetti | `GateSet::rigetti()` | rx, rz, cz |
| IonQ | `GateSet::ionq()` | rx, ry, rz, xx |
| Neutral Atom | `GateSet::neutral_atom()` | rz, rx, ry, cz |
| Simulator | `GateSet::universal()` | All standard gates |

## License

Apache License 2.0. See [LICENSE](LICENSE).
