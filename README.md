<div align="center">

# 🛰 Space Communication Priority System

<em>Mission-critical, priority-aware satellite communication framework built in Rust</em>

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Embassy](https://img.shields.io/badge/Runtime-Embassy-green)](https://embassy.dev/)
[![CCSDS](https://img.shields.io/badge/Protocol-CCSDS%20133.0--B--2-blue)](https://public.ccsds.org/pubs/133x0b2e1.pdf)
[![HMAC-SHA256](https://img.shields.io/badge/Auth-HMAC--SHA256-brightgreen)](https://tools.ietf.org/html/rfc2104)
[![heapless](https://img.shields.io/badge/Memory-heapless-lightgrey)](https://docs.rs/heapless)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Command Flow](#command-flow)
- [Priority Distribution](#priority-distribution)
- [Technology Stack](#technology-stack)
- [Frequency Bands](#frequency-bands)
- [Advanced RF Techniques](#advanced-rf-techniques)
- [Setup & Installation](#setup--installation)
- [Running the Tests](#running-the-tests)
- [Development Roadmap](#development-roadmap)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

The **Space Communication Priority System** is a Rust workspace implementing a standards-compliant,
real-time command and telemetry framework for satellite–ground communication. It models five
frequency bands with physics-based atmospheric propagation, encodes packets in the CCSDS Space
Packet Protocol, authenticates commands with HMAC-SHA256, and dispatches them through a
`heapless` priority queue with five priority tiers and TTL enforcement.

The workspace targets both embedded `no_std` environments (ARM Cortex-M via Embassy) and standard
Linux/macOS ground-station software.

> [!NOTE]
> All packet construction auto-computes CRC-16/CCITT-FALSE at the point of `SpacePacket::new()`.
> Callers cannot produce an unauthenticated packet by omission.

> [!TIP]
> Emergency-priority messages have a maximum acceptable latency of **1 ms**.
> Use `pop_valid(current_time_secs)` instead of `pop()` to enforce TTL expiry
> so stale safety-critical commands are never executed.

> [!IMPORTANT]
> The satellite crate targets `thumbv7em-none-eabihf` (ARM Cortex-M4/M7 with FPU). Add this
> target before attempting a cross-compile: `rustup target add thumbv7em-none-eabihf`.

---

## Key Features

| Feature | Status | Description |
|---|---|---|
| CCSDS Space Packet Protocol | ✅ Stable | Header + PEC per CCSDS 133.0-B-2 |
| Auto-CRC on construction | ✅ Stable | Every `SpacePacket::new()` instantly verifiable |
| 5-Tier Priority Queue | ✅ Stable | Emergency → Critical → High → Medium → Low |
| TTL-enforced dequeue | ✅ Stable | `pop_valid()` discards expired messages silently |
| HMAC-SHA256 authentication | ✅ Stable | `CommandAuthenticator::sign / verify` (FIPS 198-1) |
| Multi-band RF simulation | ✅ Stable | UHF / S / X / K / Ka + ITU-R P.618 rain model |
| Band scoring engine | ✅ Stable | `score_bands_for_conditions()` — single-pass ranking |
| Atmospheric ITU-R P.838-3 | ✅ Stable | Compile-time coefficient table, O(17) scan |
| CRC-16 lookup table | ✅ Stable | 512-byte ROM table, ~8× faster than bit-loop |
| No-heap satellite crate | ✅ Stable | Embassy async, fully `no_std`, Cortex-M target |
| FSPL link-budget helper | ✅ Stable | `BandType::free_space_path_loss_db(distance_km)` |
| Full test suite | ✅ Stable | Unit + integration + stress + simulation tests |
| Beamforming simulation | ✅ Stable | Phased-array gain, HPBW, null steering (analog/digital/adaptive) |
| MIMO spatial multiplexing | ✅ Stable | Shannon capacity with Foschini formula, spatial streams, diversity |
| DSSS spread spectrum | ✅ Stable | Processing gain, jamming margin, LPI spectral density |
| FHSS anti-jam | ✅ Stable | 128-channel hopping, jam-hit probability, throughput factor |
| Adaptive Modulation & Coding | ✅ Stable | Link-adaptive MCS selection (BPSK → 256-QAM + FEC) |
| Polarization diversity | ✅ Stable | Dual-pol isolation, Faraday rotation loss, capacity multiplier |

---

## Architecture

The workspace is divided into four crates with a strict dependency hierarchy:

```mermaid
flowchart TD
    SHARED([space-comms-shared\nsecurity · messaging · CCSDS\ntelemetry · time · types])

    SAT([satellite\nEmbassy async · no_std\nARM Cortex-M])
    GND([ground\nStd · tokio · TLS])
    SIM([frequency-band-simulation\nITU-R P.618/838 · band scoring])

    SHARED --> SAT
    SHARED --> GND
    SHARED --> SIM

    subgraph Embedded
        SAT
    end

    subgraph Host
        GND
        SIM
    end
```

### Crate Responsibilities

| Crate | Targets | Responsibility |
|---|---|---|
| `space-comms-shared` | `std` + `no_std` | Protocol types, queue, security, CCSDS, telemetry |
| `satellite` | `thumbv7em-none-eabihf` | Embassy firmware, HW abstraction, RF driver |
| `ground` | Linux/macOS | Uplink/downlink server, TLS, mission control |
| `frequency-band-simulation` | Linux/macOS | Frequency band simulation, link budget, band ranking |

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Command Flow

The following sequence shows a complete ground-to-satellite command cycle including
authentication, priority queuing, TTL-enforced dequeue, CCSDS encapsulation, and telemetry ack:

```mermaid
sequenceDiagram
    participant OPS as Mission Ops
    participant GND as Ground Station
    participant SIM as Band Selector
    participant SAT as Satellite

    OPS->>GND: Issue command (REACTION_WHEEL:SPIN:3000RPM)
    GND->>GND: HMAC-SHA256 sign with mission key
    GND->>SIM: score_bands_for_conditions(params, env)
    SIM-->>GND: [Ka-Band ✓, X-Band ✓, ... UHF ✓] sorted
    GND->>GND: PriorityQueue::push (Critical, TTL=60s)
    GND->>GND: pop_valid(now) → Critical message dequeued
    GND->>SAT: SpacePacket::new(Command, APID=0x001, seq++, payload) → CRC auto-set
    SAT->>SAT: verify_crc() == true
    SAT->>SAT: CommandAuthenticator::verify(key, msg, tag)
    SAT-->>GND: TelemetryPacket ACK (X-Band uplink)
    GND-->>OPS: ACK confirmed
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Priority Distribution

Typical mission command mix across priority tiers (from stress-test scenario data):

```mermaid
pie title Command Priority Distribution (Mission Scenario)
    "Emergency (≤1 ms)" : 5
    "Critical (≤10 ms)" : 10
    "High (≤100 ms)" : 15
    "Medium (≤1 s)" : 40
    "Low (≤10 s)" : 30
```

> [!WARNING]
> The queue has a **fixed compile-time capacity** (e.g., `PriorityQueue::<64>::new()`).
> Pushing beyond capacity returns `Err` — never panics. Size must be chosen at compile time
> for embedded targets. Stack-overflow risk is zero; heap allocation is zero.

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Technology Stack

The table below summarises every dependency and its role. Detailed explanations of the design
decisions behind each choice follow.

| Technology | Version | Crate(s) | Role |
|---|---|---|---|
| Rust | 1.70+ | all | Systems language — memory safety + `no_std` |
| Embassy | 0.2+ | `satellite` | Embedded async executor on Cortex-M |
| heapless | 0.7 | `shared`, `satellite` | Allocation-free data structures |
| HMAC + SHA-2 | 0.12 / 0.10 | `shared` | FIPS 198-1 command authentication |
| serde | 1.0 | `shared`, `simulation` | Packet serialisation / deserialisation |
| tokio | 1.36 | `ground` | Async I/O for the ground-station process |
| rustls | 0.21 | `ground` | Memory-safe TLS for relay links |
| rand | 0.8 | `simulation` | Stochastic atmospheric model |
| criterion | 0.5 | dev | Statistical micro-benchmarks |
| approx | 0.5 | dev | ULP-precise float assertions |

---

### Rust

**Why:** Space and defence systems demand both correctness guarantees and low-level hardware
access. Rust provides compile-time memory safety (no null-pointer dereferences, no
use-after-free, no data races) without a garbage collector — critical because garbage-collection
pauses are incompatible with the 1 ms Emergency-priority latency budget. The `no_std` mode lets
the same codebase compile for a bare-metal satellite microcontroller and a Linux ground server.

**What it does here:** Every crate is written in Rust. The `shared` crate compiles under both
`std` and `no_std` feature flags, controlled by `Cargo.toml`:

```toml
[features]
default = ["std"]
std     = []
no-std  = ["heapless/ufmt-impl"]
```

**How it does it:** The workspace-level `Cargo.toml` sets `unsafe_code = "forbid"` as a lint,
making any `unsafe` block a compile error. The borrow checker statically proves that the
priority queue can never be mutated from two threads simultaneously without explicit
synchronisation. The `const fn` feature is used for compile-time CRC table generation
(`CRC16_TABLE: [u16; 256]`) and for the ITU-R P.838-3 coefficient array — zero runtime cost.

---

### Embassy

**Why:** The satellite firmware requires concurrent tasks — reading sensors, servicing the RF
transceiver, running a watchdog timer — but the target microcontroller (ARM Cortex-M4/M7) has
no operating system. Traditional RTOS approaches require dynamic heap allocation and non-trivial
porting effort. Embassy provides a cooperative async executor that runs entirely in static memory.

**What it does here:** The `satellite` crate's `main.rs` is an Embassy entry point. Each
hardware interaction (UART telemetry read, SPI transceiver write, watchdog kick) becomes an
`async fn` that can yield to other tasks without blocking the processor.

**How it does it:** Embassy uses Rust's `async`/`await` syntax compiled down to state machines
by `rustc`. There is no heap; all task state lives in statically-sized stack frames. The
executor polls futures in a cooperative loop, waking them only when hardware interrupts fire —
meaning the CPU is idle (low-power sleep) whenever no work is pending. The target triple
`thumbv7em-none-eabihf` tells the linker there is no OS system call layer.

---

### heapless

**Why:** On the satellite's Cortex-M target there is no allocator — `alloc` is not available.
The priority queue that dispatches commands must still hold up to _N_ messages ranked by
priority, without ever calling `malloc`. A standard `BinaryHeap<T>` from `std` is unusable.

**What it does here:** `heapless::BinaryHeap<PriorityMessage, Max, N>` backs the
`PriorityQueue<N>` struct in `shared/src/messaging.rs`. All packet payload buffers
(`heapless::Vec<u8, 2048>`), command parameter lists (`heapless::Vec<u8, 256>`), and alert
description strings (`heapless::String<256>`) use heapless types throughout the shared crate.

**How it does it:** `heapless` stores its elements in a fixed-size array on the stack or in a
`static`. The binary heap invariant (parent ≥ children) is maintained by standard sift-up /
sift-down operations, identical to `std::BinaryHeap` but bounded at compile time. The generic
parameter `N` makes the capacity a type-level constant, so exceeding it is a compile-time or
explicit `Err` at runtime — never a panic, never undefined behaviour.

---

### HMAC + SHA-2 (`hmac` + `sha2` crates)

**Why:** Commands uplinked to a spacecraft must be authenticated. An adversary who can inject
an unauthenticated `EmergencyAbort` command could destroy a mission. A message authentication
code (MAC) proves both the identity of the sender (they hold the shared secret key) and the
integrity of the payload (any bit-flip invalidates the tag). HMAC-SHA256 is the FIPS 198-1
standard algorithm, widely used in aerospace command authentication.

**What it does here:** `CommandAuthenticator` in `shared/src/security.rs` wraps both crates.
`sign(key, message) -> AuthTag` computes the 32-byte HMAC tag. `verify(key, message, tag) ->
bool` recomputes the tag and compares in constant time (no early-exit timing leak).

**How it does it:** HMAC-SHA256 pads the key to the SHA-256 block size (64 bytes), XORs it
with the `ipad` constant, hashes the concatenation with the message, then XORs the key with
`opad` and hashes again — a two-pass construction that prevents length-extension attacks.
Both crates are `no_std` compatible, so authentication runs on the satellite firmware as well
as on the ground server. An empty key is explicitly rejected (`Err`) to prevent trivially weak
authentication.

---

### serde

**Why:** Packets and telemetry frames must be serialised to bytes for transmission over the RF
link and deserialised on the other end. Hand-rolling encoding logic for every struct is
error-prone and introduces deserialization bugs — a major source of mission failures. `serde`
provides a derive-macro approach that keeps the canonical type definition and its serialisation
logic co-located.

**What it does here:** Every public type in `shared` (`Message`, `SpacePacket`,
`TelemetryPacket`, `SpaceCommand`, `HealthStatus`, etc.) derives `Serialize + Deserialize`.
The CCSDS `SpacePacket::to_bytes()` calls `header.to_bytes()` (manual big-endian encoding per
the standard) and then appends the `data` field — with serde used for higher-level diagnostic
and logging serialisation via `serde_json`.

**How it does it:** The `#[derive(Serialize, Deserialize)]` macro generates trait
implementations at compile time. No reflection, no vtables, no runtime type information — the
generated code is equivalent to hand-written match statements over each field. This makes
encode/decode zero-overhead on release builds.

---

### tokio

**Why:** The ground station must handle multiple simultaneous connections: an uplink session
receiving commands from mission control, a downlink session receiving telemetry from the
satellite, optionally a relay to a secondary ground station, and an HTTP health endpoint.
Blocking one of these on a slow I/O call would stall the entire loop. An async runtime
multiplexes all connections onto a small thread pool without the overhead of one-thread-per-connection.

**What it does here:** The `ground` crate's `main.rs` runs under the `#[tokio::main]` macro.
TCP accept loops, TLS handshakes, and CCSDS frame reads are all `async fn` awaited inside
tokio tasks. The `PriorityQueue` is wrapped in a `tokio::sync::Mutex` so uplink and downlink
tasks share it safely.

**How it does it:** tokio uses an M:N scheduler — M async tasks mapped onto N OS threads
(typically one per CPU core). I/O readiness is driven by `epoll` on Linux. When a socket has
no data, the task is parked (no CPU consumed) until the kernel wakes it via an event. This
gives the ground station the ability to sustain hundreds of concurrent connections with memory
proportional to active work, not total connections.

---

### rustls

**Why:** Commands and telemetry transiting the RF-to-internet relay leg must be encrypted and
authenticated at the transport layer to prevent eavesdropping and man-in-the-middle injection.
OpenSSL is the traditional choice but it is written in C, has a long history of memory-safety
CVEs (Heartbleed, etc.), and is difficult to audit. `rustls` provides TLS 1.3 implemented
entirely in safe Rust.

**What it does here:** The ground-to-relay TCP connection in the `ground` crate wraps its
`TcpStream` in a `rustls::StreamOwned<ClientConnection, TcpStream>`, giving transparent
encryption over all CCSDS packet bytes transmitted to the relay server.

**How it does it:** `rustls` implements TLS 1.3 handshake (key exchange via X25519 ECDH,
authentication via Ed25519 certificates), bulk encryption (AES-256-GCM), and record-layer
framing in pure safe Rust. It uses the `ring` crate for cryptographic primitives. Because
there is no `unsafe` in `rustls` itself, entire classes of buffer-overflow and
use-after-free vulnerabilities are structurally impossible.

---

### rand

**Why:** The atmospheric simulation must model stochastic weather phenomena — rain-cell
distribution, scintillation variance, ionospheric turbulence — that are inherently random.
A deterministic sine-wave approximation would under-sample worst-case conditions used in
link-budget margin planning.

**What it does here:** `simulation/src/atmospheric.rs` uses `rand` to sample rain-rate
distributions for Monte Carlo link-availability predictions and to generate synthetic
scintillation time-series for testing fade-margin calculations.

**How it does it:** `rand` separates the RNG algorithm (`ChaCha8Rng` — fast, cryptographically
secure) from the sampling layer (`distributions::Normal`, `distributions::Uniform`). The
simulation seeds the RNG from a fixed seed in tests (deterministic) and from
`rand::thread_rng()` (OS entropy) in production runs.

---

### criterion (dev)

**Why:** The CRC-16 lookup-table optimisation and the single-pass band-scoring refactor
(`score_bands_for_conditions`) each claim a significant performance improvement. Without a
statistical benchmarking framework, "it feels faster" is not a measurable claim.

**What it does here:** Micro-benchmarks in the `benches/` directory measure CRC throughput
(bytes/second before and after the lookup table) and band-scoring latency across queue sizes.

**How it does it:** criterion runs each benchmark in a calibrated loop, collects wall-clock
samples, computes mean ± standard deviation, and reports regression against a previously saved
baseline. It uses Welch's t-test to distinguish genuine regressions from measurement noise.

---

### approx (dev)

**Why:** The link-budget calculations (`free_space_path_loss_db`, rain attenuation, SNR) produce
floating-point results. Comparing floats with `==` is incorrect due to rounding; comparing with
a hard-coded epsilon is fragile across platforms. `approx` provides ULP (units in the last place)
and relative-epsilon assertion macros standardised for IEEE 754.

**What it does here:** Simulation integration tests use `assert_relative_eq!(fspl, expected,
epsilon = 0.01)` to assert that band-scoring results are within 0.01 dB of the expected
link-budget value, regardless of compiler optimisation level or FPU rounding mode.

**How it does it:** `approx::relative_eq!(a, b, epsilon = e)` computes
`|a − b| / max(|a|, |b|) < e`, which is scale-invariant and correct for both very large and
very small floating-point values — unlike an absolute epsilon which would pass accidentally for
large numbers or fail spuriously for small ones.

---

## Frequency Bands

The simulation crate models five ITU standardised bands.
`score_bands_for_conditions()` performs a single-pass evaluation and returns all bands
ranked by composite utility score (0 = unusable, 1 = optimal):

| Band | Range | Max Rate | Rain Sensitivity | Best For |
|---|---|---|---|---|
| **UHF** | 0.3–3 GHz | 10 Mbps | Very Low | Emergency backup, all-weather ops |
| **S-Band** | 2–4 GHz | 100 Mbps | Low | TT&C, routine telemetry |
| **X-Band** | 8–12 GHz | 500 Mbps | Medium | Science data downlink |
| **K-Band** | 20–30 GHz | 1 Gbps | High | High-rate data relay |
| **Ka-Band** | 26.5–40 GHz | 2 Gbps | Very High | Maximum bandwidth, clear sky |

```mermaid
flowchart LR
    ENV{Weather\nConditions}
    ENV -->|Clear sky| KA["🔵 Ka-Band\n2 Gbps"]
    ENV -->|Light rain| KB["🟢 K-Band\n1 Gbps"]
    ENV -->|Heavy rain| XB["🟡 X-Band\n500 Mbps"]
    ENV -->|Severe storm| SB["🟠 S-Band\n100 Mbps"]
    ENV -->|Tropical storm| UHF["🔴 UHF\n10 Mbps"]
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Advanced RF Techniques

The `simulation/src/advanced_rf.rs` module implements six advanced radio-frequency techniques
that complement the basic band selection model. Each technique is independently configurable and
can be combined to model a complete waveform stack for a mission-critical satellite link.

### 1. Beamforming

**What it is:** A phased-array technique that steers a focused RF beam toward the target satellite
or ground station by applying precise phase (and optionally amplitude) weights to each antenna
element, rather than relying on a single omnidirectional radiator.

**How it works:** For a Uniform Linear Array (ULA) of *N* elements at half-wavelength spacing,
all elements combine coherently in the desired direction, producing an array gain of
$G = 10 \cdot \log_{10}(N)$ dBi and a half-power beamwidth of
$\theta_{3\text{dB}} \approx 0.886 / (N \cdot d/\lambda)$ radians.
Null steering places a pattern zero toward an interference source.

**Three modes supported:**

| Mode | Null Depth | Notes |
|---|---|---|
| Analog | ~25 dB | Phase shifters only; hardware quantisation limits nulls |
| Digital | ~40 dB | Per-element baseband weights after ADC |
| Adaptive (MVDR) | ~55 dB | Continuously optimised weights track moving interferers |

**Satellite relevance:** Ka-Band high-gain links require narrow beams (≤ 1°) with active null
steering to suppress adjacent-satellite interference during station-keeping manoeuvres.

### 2. MIMO (Multiple Input Multiple Output)

**What it is:** Use of multiple antennas at both ends of a link to transmit independent data
streams (spatial multiplexing) or the same stream redundantly (transmit diversity), improving
either capacity or reliability without additional spectrum or power.

**How it works:** With *N_s* = min(*N_tx*, *N_rx*) independent channels in a rich scattering
environment, the Shannon–Foschini capacity is:

$$C = N_s \cdot B \cdot \log_2\!\left(1 + \frac{\text{SNR}}{N_s}\right)$$

For transmit diversity (Alamouti STBC), the diversity order is $N_{tx} \times N_{rx}$ and the
combined SNR gain is $10 \cdot \log_{10}(N_{tx} \cdot N_{rx})$ dB.

**Three modes supported:** `SpatialMultiplexing`, `TransmitDiversity`, `MassiveMimo`
(large-scale antenna array with zero-forcing precoding, scales capacity as N_rx streams).

**Satellite relevance:** High-throughput satellites (HTS) use massive MIMO on Ka-Band gateways
to simultaneously serve hundreds of user beams from a single dish array.

### 3. DSSS — Direct-Sequence Spread Spectrum

**What it is:** A modulation technique that XORs the information signal with a high-rate
pseudo-noise (PN) spreading code, deliberately spreading the signal energy across a bandwidth
many times wider than the raw data rate.

**How it works:** The processing gain $G_p = f_{chip} / f_{data}$ (linear) directly translates
to a jamming margin:
$$\text{Jam Margin} = G_p - E_b/N_0^{\min} \quad (\text{dB})$$
The 1023-chip Gold code used in the demo gives $G_p \approx 30.1$ dB, meaning a jammer must
be 30 dB stronger than the signal to overcome the despreader — and the transmission appears as
noise below the noise floor to any non-synchronised receiver (LPI property).

**Satellite relevance:** GPS and military SATCOM (MILSTAR, AEHF) rely on DSSS for anti-jam
resilience. GPS L1 C/A uses a 1.023 Mcps chipping rate over a 1.023 kbps navigation message.

### 4. FHSS — Frequency-Hopping Spread Spectrum

**What it is:** The transmitter rapidly changes carrier frequency according to a
cryptographically generated pseudo-random sequence shared only with the intended receiver.
An interceptor or jammer must cover all possible channels simultaneously to be effective.

**How it works:** With *N* channels of bandwidth *B_c*, the total spread bandwidth is
$N \cdot B_c$ and the processing gain is $10 \cdot \log_{10}(N)$ dB.
- **Fast hopping** (multiple hops per symbol): symbol energy spread across several channels;
  a partial-band jammer hitting one channel only corrupts part of the symbol.
- **Slow hopping** (multiple symbols per hop): simpler implementation; vulnerable to
  follower jamming if the jammer can track hop transitions.

Probability a given hop hits a jammed channel: $p_{jam} = N_{jammed} / N_{total}$.

**Satellite relevance:** Military UHF SATCOM (MILSTAR LDR/MDR channels) use FHSS for
anti-intercept communications. 128-channel fast hopping at 1000 hops/s effectively denies
an adversary any single coherent interception opportunity.

### 5. Adaptive Modulation and Coding (AMC / ACM)

**What it is:** The link continuously measures SNR and selects the highest-order modulation
scheme and FEC code rate whose required $E_b/N_0$ (with margin) is met by the current link
quality, maximising spectral efficiency under varying propagation conditions.

**Supported MCS ladder:**

| Modulation | Bits/Symbol | Min Eb/N0 (BER < 10⁻⁶) |
|---|---|---|
| BPSK | 1 | 10.5 dB |
| QPSK | 2 | 10.5 dB |
| 16-QAM | 4 | 14.5 dB |
| 64-QAM | 6 | 18.8 dB |
| 256-QAM | 8 | 24.0 dB |

**FEC rates supported:** 1/2 · 2/3 · 3/4 · 5/6 · 7/8 (coding gains 0.5–3.5 dB).

The algorithm walks from 256-QAM + rate 7/8 downward and selects the first pair with
≥ 2 dB link margin, ensuring a guard band against sudden fades.

**Satellite relevance:** DVB-S2X (used by commercial HTS and ESA/NASA relay satellites)
employs ACM to adapt per-frame MCS based on return-channel SNR estimates, delivering
up to 6× spectral efficiency improvement over fixed QPSK/rate-1/2 waveforms.

### 6. Polarization Diversity

**What it is:** Use of orthogonal electromagnetic field orientations (horizontal/vertical linear
or right/left circular) on the same frequency to double available channel capacity via
polarization reuse, or to improve link resilience by combining both polarizations.

**How it works:** Two orthogonal polarizations are theoretically isolated by infinite dB; in
practice, antenna cross-polarization discrimination (XPD) limits isolation to 25–35 dB for
well-designed feeds. A dual-polarization link doubles capacity when:
$$\text{XPD} \geq 20 \text{ dB and polarization mismatch loss} < 1 \text{ dB}$$

Faraday rotation in the ionosphere rotates the polarization plane by an angle proportional
to the Total Electron Content (TEC) and inversely proportional to frequency squared —
only significant below ~3 GHz (UHF/S-Band); negligible at X/Ka-Band.

**Satellite relevance:** Intelsat and SES HTS satellites use dual-linear (H+V) polarization
reuse to serve twice as many beams per transponder. Circular polarization (RHCP/LHCP)
is preferred on LEO science downlinks to avoid Faraday rotation alignment requirements.

### Combined Advanced-RF Summary (LEO Science Link, X-Band)

Running `cargo run -p frequency-band-simulation` produces a summary of all six techniques
applied together to a representative 500 km LEO science downlink:

```
1. BEAMFORMING  — 16-element ULA, Adaptive: 12.0 dBi gain, 3.17° HPBW, 55 dB null
2. MIMO         — 4×4 Spatial MX, 30 dB SNR: 4 streams, ~4600 Mbps, 4× SISO gain
3. DSSS         — 1023-chip Gold, 10 Mcps: 30.1 dB Gp, jam-resistant ✓
4. FHSS         — 128 ch fast hop, 5% jammed: 21.1 dB Gp, 97% throughput, High rating
5. AMC          — 30 dB SNR → 256-QAM r=7/8: ~3500 Mbps, 7.0 bps/Hz, 8.5 dB margin
6. POLARIZATION — Dual-Circular, 30 dB XPD: 2× capacity, 0.06 dB pol loss
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Setup & Installation

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Embedded cross-compile target (satellite crate only):
  ```bash
  rustup target add thumbv7em-none-eabihf
  ```

### Build

```bash
cd rust-workspace

# Build all workspace members
cargo build --workspace

# Build only the shared library
cargo build -p space-comms-shared

# Build only the simulation library
cargo build -p frequency-band-simulation

# Cross-compile satellite firmware (requires Cortex-M target)
cargo build -p satellite --target thumbv7em-none-eabihf
```

### Run Demonstrations

```bash
# Frequency-band simulation demo (clear-sky vs storm band ranking)
cargo run -p frequency-band-simulation

# Ground station (requires running satellite simulation)
cargo run -p ground
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Running the Tests

The project ships four test suites:

### Quick test all

```bash
cd rust-workspace
cargo test --workspace
```

### Per-crate tests

```bash
# Shared library — unit + integration + security + CCSDS + TTL enforcement
cargo test -p space-comms-shared

# Simulation crate — band scoring, ITU-R model, link budgets
cargo test -p frequency-band-simulation
```

### Test Coverage Summary

<details>
<summary>Show test matrix (click to expand)</summary>

| Test File | Crate | Test Count | Coverage Area |
|---|---|---|---|
| `shared/tests/full_suite.rs` | `space-comms-shared` | ~55 tests | CCSDS, queue, security, bands, time, telemetry, errors |
| `shared/tests/integration_tests.rs` | `space-comms-shared` | ~35 tests | End-to-end lifecycle, full command pipeline |
| `simulation/tests/simulation_tests.rs` | `frequency-band-simulation` | ~22 tests | Band scoring, physics invariants, determinism |
| `tests/priority_stress_tests.rs` | reference | — | Async stress, mission scenario playbooks |

</details>

<details>
<summary>Show key test invariants (click to expand)</summary>

- **Auto-CRC**: `SpacePacket::new()` → `verify_crc() == true` (no additional call needed)
- **TTL enforcement**: expired Emergency silently discarded; valid Low surfaces via `pop_valid()`
- **Priority order**: Emergency → Critical → High → Medium → Low always preserved by heap
- **HMAC security**: tampered message/wrong key both produce `verify() == false`
- **Band scoring**: Ka-Band wins in clear sky; UHF wins in 100 mm/hr storm
- **FSPL monotonicity**: FSPL increases with distance and with centre frequency
- **Queue overflow**: `push()` beyond capacity returns `Err`, never panics

</details>

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Development Roadmap

```mermaid
gantt
    title Space Communication Priority System — Roadmap
    dateFormat  YYYY-MM-DD
    section Foundation
    Shared types + CCSDS           :done,   f1, 2024-10-01, 2024-11-01
    Priority queue + TTL           :done,   f2, 2024-10-15, 2024-11-15
    HMAC-SHA256 authentication     :done,   f3, 2024-11-01, 2024-12-01
    section Optimisations
    CRC-16 lookup table            :done,   o1, 2024-11-15, 2024-12-01
    ITU-R P.838-3 coefficient table:done,   o2, 2024-11-20, 2024-12-10
    Band scoring single-pass       :done,   o3, 2024-12-01, 2024-12-15
    FSPL link-budget helper        :done,   o4, 2024-12-01, 2024-12-15
    section Testing
    Full unit + integration tests  :done,   t1, 2025-01-01, 2025-02-01
    Simulation test suite          :done,   t2, 2025-01-15, 2025-02-15
    section Advanced RF
    Beamforming + MIMO simulation   :done,   a1, 2025-03-01, 2026-04-08
    DSSS + FHSS spread spectrum     :done,   a2, 2025-03-15, 2026-04-08
    AMC link-adaptive waveforms     :done,   a3, 2025-04-01, 2026-04-08
    Polarization diversity model    :done,   a4, 2025-04-01, 2026-04-08
    section Upcoming
    AES-256-GCM payload encryption :active, u1, 2025-02-01, 2025-04-01
    Carrier Doppler correction      :        u2, 2025-03-01, 2025-05-01
    Hardware-in-loop integration    :        u3, 2025-04-01, 2025-07-01
```

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Project Structure

```
rust-workspace/
├── Cargo.toml                  # Workspace manifest
├── shared/                     # Core shared library (std + no_std)
│   ├── src/
│   │   ├── lib.rs              # Module root + re-exports
│   │   ├── ccsds.rs            # CCSDS Space Packet + CRC-16 lookup table
│   │   ├── messaging.rs        # PriorityQueue, Message, pop_valid()
│   │   ├── security.rs         # CommandAuthenticator (HMAC-SHA256)
│   │   ├── telemetry.rs        # TelemetryPacket + measurements
│   │   ├── commands.rs         # 25-command SpaceCommand catalog
│   │   ├── types.rs            # BandType, ComponentId, HealthStatus, FSPL
│   │   ├── error.rs            # SpaceCommError + MemoryErrorType
│   │   └── time.rs             # Timestamp, Duration, current_time_nanos()
│   └── tests/
│       ├── full_suite.rs       # Comprehensive unit + module tests (~55 tests)
│       └── integration_tests.rs# End-to-end integration tests (~35 tests)
├── simulation/                 # RF propagation simulation (std only)
│   ├── src/
│   │   ├── lib.rs              # FrequencyBand, BandScore, score_bands_for_conditions()
│   │   ├── advanced_rf.rs      # Beamforming, MIMO, DSSS, FHSS, AMC, polarization diversity
│   │   ├── atmospheric.rs      # RainFadeModel — ITU-R P.618/838
│   │   ├── bands.rs            # Extended band definitions
│   │   └── interference.rs     # Interference + Doppler models
│   └── tests/
│       └── simulation_tests.rs # Band scoring + physics invariants (~22 tests)
├── satellite/                  # Embedded firmware (no_std, Cortex-M)
│   └── src/
│       ├── main.rs             # Embassy async entry point
│       ├── communication.rs    # RF + protocol driver
│       └── hardware.rs         # HAL abstraction
├── ground/                     # Ground station daemon (std + tokio)
│   └── src/
│       └── main.rs             # Uplink/downlink server
└── tests/                      # Workspace-level reference test files
    ├── integration_tests.rs    # System integration reference
    └── priority_stress_tests.rs# Async stress + mission scenario playbooks

config/
├── requirements.txt            # Python runtime dependencies
└── requirements-dev.txt        # Python development dependencies

scripts/
└── deprecated/                 # One-time migration scripts (archived)
```

<details>
<summary>Show full command catalog (click to expand)</summary>

The `SpaceCommand` enum in `shared/src/commands.rs` defines **25 mission commands**
classified across five priority tiers:

| Priority | Command | Latency Requirement |
|---|---|---|
| Emergency | `EmergencyAbort` | ≤1 ms |
| Emergency | `EmergencyHalt` | ≤1 ms |
| Emergency | `ActivateSafeMode` | ≤1 ms |
| Critical | `AbortMission` | ≤10 ms |
| Critical | `CollisionAvoidance` | ≤10 ms |
| Critical | `AttitudeControl` | ≤10 ms |
| Critical | `PowerManagement` | ≤10 ms |
| High | `UpdateOrbit` | ≤100 ms |
| High | `ReconfigureComm` | ≤100 ms |
| High | `DeployInstrument` | ≤100 ms |
| High | `AdjustAntenna` | ≤100 ms |
| High | `ThermalControl` | ≤100 ms |
| Medium | `RequestTelemetry` | ≤1 s |
| Medium | `UpdateConfiguration` | ≤1 s |
| Medium | `CalibrateInstrument` | ≤1 s |
| Medium | `StartDataCollection` | ≤1 s |
| Medium | `StopDataCollection` | ≤1 s |
| Low | `SendStatusReport` | ≤10 s |
| Low | `UpdateTimeSync` | ≤10 s |
| Low | `PerformMaintenance` | ≤10 s |
| Low | `ScheduleActivity` | ≤10 s |
| Low | `DownloadLogs` | ≤10 s |
| Low | `UpdateSoftware` | ≤10 s |
| Low | `DiagnosticScan` | ≤10 s |
| Low | `ArchiveData` | ≤10 s |

</details>

<div align="right"><a href="#table-of-contents">↑ Back to top</a></div>

---

## Contributing

Contributions are welcome. Please follow these guidelines:

1. Follow the existing Rust formatting: `cargo fmt --all`
2. Ensure `cargo clippy --workspace -- -D warnings` passes
3. Add structured mission-critical comments to any new public functions
4. All new public API must have integration tests in the appropriate `tests/` directory
5. Do not disable `unsafe_code = "forbid"` — it is a workspace-level lint

> [!CAUTION]
> This project enforces `#[forbid(unsafe_code)]` workspace-wide. Pull requests introducing
> `unsafe` blocks will not be accepted regardless of justification.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">
<sub>Built with precision for mission-critical space systems.</sub>
</div>
