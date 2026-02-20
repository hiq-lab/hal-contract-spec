# HAL Contract — Backend Integration Roadmap

**Last updated:** 2026-02-20

## Current Integrations

Seven production-ready adapters ship with [Arvak](https://arvak.io), the reference implementation:

| Adapter | QPUs | Technology | Access |
|---------|------|------------|--------|
| **Local Simulator** | Statevector (up to ~25q) | Pure Rust | None |
| **IBM Quantum** | Eagle (127q), Heron (156q), Nighthawk (120q) | Superconducting | API token |
| **IQM** | Resonance cloud, LUMI Helmi, LRZ Garching | Superconducting | Token / OIDC |
| **Scaleway QaaS** | IQM Garnet (20q), Emerald (54q), Sirius (16q) | Superconducting | API key |
| **AWS Braket** | Rigetti Ankaa-3, IonQ Aria/Forte, IQM, SV1/TN1/DM1 | Multi-vendor | AWS credentials |
| **NVIDIA CUDA-Q** | cuStateVec, TensorNet, Density Matrix (GPU) | Simulation | API token |
| **QDMI (MQSS)** | Any QDMI v1.2.1+ device at LRZ/JSC | Standard FFI | Token / OIDC |

## Phase 1 — Quantinuum (H2 / Helios)

**Target:** Q1 2026
**Priority:** Highest

Quantinuum operates the highest-fidelity quantum hardware commercially available. Helios (3rd generation) delivers 98 barium-ion qubits at 99.92% two-qubit entanglement fidelity — the most accurate quantum computer in the world as of late 2025. The H2 system holds the Quantum Volume record at 33,554,432.

**Why it matters for HAL Contract:**
- All-to-all connectivity (no SWAP routing overhead)
- Mid-circuit measurement and conditional logic (classical feedforward)
- Qubit reuse — fundamentally different execution model than superconducting
- Proves HAL Contract works at the quality frontier, not just the scale frontier

**Technical details:**
- Native gates: Rz, U1q (single-qubit), ZZPhase (two-qubit)
- Trapped-ion architecture, ytterbium (H2) / barium (Helios) ions
- T2 coherence times ~seconds (1000x longer than superconducting)
- Supports OpenQASM, pytket/TKET compiler

**Access path:** Azure Quantum REST API ($500 free credits per provider) or pytket-quantinuum direct SDK. Cambridge UK company (Honeywell/Cambridge Quantum Computing spin-off).

**Adapter approach:** Azure Quantum unified REST adapter, or pytket-based FFI integration for native compilation support.

**Roadmap:** Sol (192 qubits, 2027), Apollo (thousands of qubits, fault-tolerant, 2029).

## Phase 2 — AQT (Alpine Quantum Technologies)

**Target:** Q2 2026
**Priority:** High

Austrian company based in Innsbruck. Hardware physically located in the EU — real data sovereignty. The IBEX Q1 system is rack-mountable (two 19-inch racks, <2kW power), specifically designed for HPC center deployment.

**Why it matters for HAL Contract:**
- EU data sovereignty — hardware in Austria
- Compact, HPC-native form factor (rack-mountable, low power)
- Trapped-ion with all-to-all connectivity
- EuroHPC tender awarded for trapped-ion installation (2025)

**Technical details:**
- IBEX Q1: 12 qubits, calcium-40 ions, fully connected
- PINE: Table-top system scaling to 50 qubits
- Native gates: MS (Molmer-Sorensen), arbitrary single-qubit rotations
- All-to-all connectivity (like Quantinuum, unlike superconducting)

**Access path:** Already reachable via existing Braket adapter (eu-north-1 Stockholm region) and Scaleway. A direct adapter enables HPC-native OIDC authentication for institutional deployments.

**Adapter approach:** Direct REST API adapter with OIDC support for HPC centers. Braket adapter provides interim coverage.

## Phase 3 — planqc (Munich)

**Target:** Q2–Q3 2026
**Priority:** High (strategic)

German company headquartered in Munich. Deploying at LRZ Garching through the Munich Quantum Software Stack (MQSS). EUR 20M BMBF-funded MAQCS project targeting a 1,000-qubit universally programmable neutral-atom quantum computer.

**Why it matters for HAL Contract:**
- German company, Munich-based — same ecosystem as HAL Contract
- LRZ Garching deployment — same HPC center, same software stack (MQSS/QDMI)
- Neutral atoms in optical lattices — multi-core architecture for speed and scalability
- 1,000-qubit target is the most ambitious European hardware roadmap

**Technical details:**
- Neutral-atom technology with optical lattice trapping
- Multi-core architecture (parallel execution zones)
- Native gates: Rydberg CZ, global rotations (TBD as API stabilizes)
- QDMI integration planned through MQSS

**Access path:** Not yet publicly accessible. Integration likely through existing QDMI adapter once planqc's QDMI driver ships. Establish early access partnership now.

**Adapter approach:** QDMI adapter (already implemented) should provide coverage once planqc's QDMI driver is available. Custom adapter if planqc exposes a direct API.

## Phase 4 — PASQAL (Paris)

**Target:** Q3 2026
**Priority:** Medium

Key European neutral-atom company based in Paris. 100+ qubit Orion Alpha system operational. 140+ qubit system delivered to Italy under EuroHPC. 250-qubit QPU targeted for H1 2026. Available on Azure Quantum, Scaleway, OVHcloud, and Google Cloud.

**Why it matters for HAL Contract:**
- Major European quantum hardware company (French)
- EuroHPC deployment in Italy (February 2026)
- Multi-cloud availability (Azure, Scaleway, OVHcloud, Google)
- Arbitrary 2D/3D atom placement — programmable topology

**Technical details:**
- Neutral-atom technology with reconfigurable atom arrays
- Currently primarily analog mode (Rydberg Hamiltonian simulation)
- Evolving toward digital gate-based mode
- SDKs: Pulser (hardware-near), Qadence (higher-level)

**Caveat:** PASQAL's current programming model is primarily analog (Hamiltonian simulation), not gate-based circuits. HAL Contract's `Backend<C>` trait assumes gate-based circuit dispatch. Integration requires either waiting for PASQAL's digital mode API or extending HAL Contract with an analog execution mode.

**Adapter approach:** Pulser-based adapter when digital mode matures. Consider an analog mode extension to the spec if demand justifies it.

## Phase 5 — Quandela (Paris)

**Target:** Q4 2026
**Priority:** Medium-low

French photonic quantum computing company. 12-qubit Belenos system operational, 24-qubit Canopus targeted for 2026. Delivered a system to CEA TGCC under EuroQCS-France. 80% of components sourced from EU suppliers.

**Why it matters for HAL Contract:**
- Photonic QC — completes EU modality coverage
- French company, EU-sourced hardware
- EuroQCS-France consortium — available to European researchers
- Room-temperature operation, telecom-compatible photons

**Technical details:**
- Linear optical quantum computing (single-photon sources + beam splitters)
- Perceval open-source SDK for photonic circuit programming
- MerLin quantum ML language (PyTorch/scikit-learn integration)
- Available on Quandela Cloud, OVHcloud (mid-2026)

**Caveat:** Photonic QC uses a fundamentally different circuit representation (linear optical networks, measurement-based computation). This is not a standard gate model — the adapter would need to translate between HAL Contract's circuit abstraction and Perceval's photonic circuit model. Small qubit counts (12–24) limit near-term utility.

**Adapter approach:** Perceval SDK wrapper with circuit translation layer. High adapter complexity relative to qubit count, but strategically important for EU ecosystem completeness.

## After the Roadmap

Following these five phases, HAL Contract would cover every commercially accessible quantum technology modality and every significant European quantum hardware provider:

| Modality | Providers |
|----------|-----------|
| **Superconducting** | IBM, IQM (+ via Scaleway, Braket), Rigetti (via Braket) |
| **Trapped ion** | Quantinuum, AQT, IonQ (via Braket) |
| **Neutral atom** | planqc, PASQAL |
| **Photonic** | Quandela |
| **Simulation** | Local, NVIDIA CUDA-Q, AWS SV1/TN1/DM1 |
| **HPC standard** | QDMI (any compliant device) |

## Excluded Providers

### D-Wave — Excluded

D-Wave builds quantum annealers, not gate-based quantum computers. Their Advantage2 system (4,400+ qubits, Zephyr topology) solves QUBO/Ising optimization problems — a fundamentally different computation model from the gate-based circuit dispatch that HAL Contract specifies. Problems are formulated as quadratic unconstrained binary optimization, not as sequences of quantum gates.

Beyond the technical mismatch, D-Wave declined partnership when approached directly (February 2026).

### Google Quantum AI — Excluded

Google's Willow processor (105 qubits) demonstrated groundbreaking exponential quantum error correction scaling in late 2024, but the platform is **not commercially accessible**. There is no self-service access, no pricing, and no cloud API. Access is restricted to internal Google research and select academic partnerships. Google is not in the quantum-as-a-service business and has given no indication of opening access before 2027–2028 at the earliest.

If Google eventually offers commercial cloud access via Cirq/Quantum Engine, a HAL Contract adapter would be straightforward to build.

### Xanadu — Excluded

Xanadu's Aurora system (12 universal photonic qubits, 35 photonic chips, 13km fiber) was announced in 2025 but is **not commercially available**. Their previous Borealis system (216 squeezed modes) performed Gaussian boson sampling — not gate-based computation. Xanadu's primary contribution to the ecosystem is PennyLane (an open-source quantum ML framework), which is hardware-agnostic and already works with backends HAL Contract supports (IBM, IonQ, Rigetti).

PennyLane could be considered as a circuit input format for HAL Contract, but Xanadu's hardware itself is not accessible for integration.

### Origin Quantum — Excluded

Origin Quantum (China) operates the 72-qubit Wukong superconducting processor and has served users from 139 countries via their cloud platform. However, data sovereignty concerns, EU–China technology export restrictions, and the complete absence of interoperability with Western quantum SDKs or European HPC infrastructure make Origin Quantum a non-starter for a German/EU quantum-HPC project. There is no standard API compatibility, no EuroHPC integration path, and significant geopolitical risk.

### Intel Quantum SDK — Excluded

Intel's Quantum SDK provides CPU-based quantum simulation with a focus on silicon quantum dot physics models aligned with Intel's (not yet commercially available) qubit technology. The simulator lacks GPU acceleration, making it uncompetitive with NVIDIA cuQuantum and Qiskit Aer for general-purpose simulation. The quantum dot simulator serves a niche research use case that does not align with HAL Contract's focus on orchestrating real quantum hardware.

### IonQ / Rigetti (direct) — Not Separate Adapters

IonQ (Aria 25q, Forte 36q, Tempo 100q coming 2026) and Rigetti (Ankaa-3 84q) are excellent hardware platforms, but both are already accessible through the existing **AWS Braket adapter**. Building separate direct adapters would duplicate functionality without adding meaningful capability. If either provider introduces HPC-native deployment (on-premises, OIDC authentication) that Braket doesn't cover, a direct adapter becomes justified.

## License

Apache License 2.0. See [LICENSE](LICENSE).
