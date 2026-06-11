# HALOS - Hardware Abstraction Layer OS

*A self-healing, zero-downtime operating system where failure is a first-class concept.*

> This project began by following Philipp Oppermann's blog-os project, then spun off into a structural rewrite leading what halos today. Many thanks to Mr. Oppermann and his wonderful resource "Writing an OS in Rust". Check it out here! https://os.phil-opp.com/

HALOS is an effort to build self-healing, zero-downtime systems. Instead of treating crashes as fatal, it treats failure as something to be detected, contained, and recovered from at runtime without rebooting. The system is built on isolated processes, typed message passing, capability-scoped authority, and supervision trees, so that one failing component never takes down the machine.

At the base sits a **minimal-policy microkernel**. The kernel's job is to implement mechanism instead of policy. An exercise in stripping down a kernel to the bare necessities, halos-kernel owns only isolation, memory management, scheduling, IPC delivery, capability validation, and epoch enforcement. Two deliberate, bounded, and explicitly acknowledged policy fragments exist within the kernel: an atomic *constructor primitive* that builds a process address space from a validated manifest, and a *Core-recovery primitive* that rebuilds the supervisor's trusted core from a known-good image. Everything else (drivers, services, supervision strategy) lives in user space.

The design and architectural decisions derive from several core principles. The primary guiding principle behind every decision is that structure should define the security model. All core principles detailed below stem from this one guiding principle.

Supervision and process orchestration are handled by **Sentinel**, a user-space supervisor that owns the authoritative process topology and is itself restartable. Sentinel splits into two layers, Core and Logic. The mechanical **Core** layer validates manifests, holds the topology, and is the only component allowed to invoke kernel primitives. The policy-bearing **Logic** layer implements supervision trees, restart strategies, and escalation. Binary parsing is confined to **Berth**, a parser that turns an ELF64 image into a fixed-shape *manifest* and holds no kernel authority of any kind, so that a parser exploit yields a rejectable manifest rather than a constructed address space.

The model draws on Erlang/OTP's "let it crash" philosophy and the minimalism of microkernels, but the isolation guarantees come from the kernel and the MMU rather than a language runtime. Processes are organized into hierarchical supervision trees, where failures are isolated and handled according to explicit, deterministic strategies.

HALOS prioritizes bounded latency, predictable failure behavior, and no global stalls. IPC overhead is a deliberate trade-off for predictability and recoverable state. The axes of evaluation are correctness, minimality, and auditability.

## Core Principles

- **Separation of mechanism and policy.** The kernel is mechanism, Sentinel-Logic is policy, and Sentinel-Core mediates and enforces invariants. Policy may evolve, fail, and be replaced; mechanism must remain correct under all circumstances. Mechanism, when privileged, must be bounded and validated at the next point in dataflow.
- **Capability-derived authority.** Every action that crosses an address-space boundary is authorized by an opaque, unforgeable capability token. There is no ambient authority anywhere, and only the kernel mints capabilities.
- **Epoch-based temporal integrity.** Every process, capability, and topology edge carries a monotonically increasing epoch. Stale or replayed authority is rejected in constant time, closing resurrection, replay, and time-of-check/time-of-use races by construction.
- **No binary parsing outside Berth, and no construction inside it.** A program binary holds no privileged status; any parser that emits a valid manifest can stand in for Berth.
- **Memory is owned, not allocated (it expires, it is not freed).** Every byte lives in a region bound to one lifecycle domain. Reclamation is bulk, at lifecycle boundaries: no fine-grained `free` used in halos-native programs, no global heap.
- **Structural safety over runtime checks.** Isolation is region ownership the hardware enforces, W^X is an enum with no representable writable-and-executable state, and temporal validity is an epoch comparison, each a property of the type surface rather than a check bolted onto a permissive substrate. Structure should serve to enforce the security model via mechanism instead of policy.
- **Explicit, typed interactions across every boundary.** There is no ambient syscall surface. Every interaction is typed, fixed-shape, capability-authorized, epoch-validated, and auditable.

## Components

- **Kernel** — minimal microkernel: scheduling, MMU and memory management, IPC delivery, capability validation, epoch enforcement, plus the two bounded structural primitives (manifest-driven construction and Core recovery). Holds no supervision policy and parses no binary format.
- **Sentinel-Core** — the mechanism of supervision. Owns the process topology, validates manifests, enforces invariants, and is the sole user-space holder of the capability to invoke kernel primitives.
- **Sentinel-Logic** — the policy of supervision. Supervision trees, restart strategies (one-for-one, one-for-all, rest-for-one), escalation, and actor semantics. Replaceable and hot-swappable, and holds no direct authority. Every change it proposes is validated by Sentinel-Core.
- **Berth** — the program parser. Transforms a program binary (of supported container format) into a validated manifest, and holds no kernel-touching capability.

A program start is three guarded steps: parse, construct, activate. Each step is followed by an independent check: Sentinel-Core's manifest validation, the kernel's cheap structural re-check, and Sentinel-Core's pre-activation invariant check. Execution begins only when the kernel activates an already-constructed address space.

## What Makes HALOS Different?

- **Failure is contained by construction instead of runtime checks.** A fault stops in hardware, the kernel reports a fact rather than an interpretation, and the blast radius is bounded by the failing process's domain. The supervisor itself is supervisable: if Sentinel-Core faults, the kernel rebuilds it from a known-good image with no loss of kernel state.
- **Stackless execution model.** Halos-native processes are not stackful. Each compiles to a state machine whose live-across-suspension state is a single bounded, position-independent *frame*. Running stacks belong to scheduler threads and are reused across processes, so a system of many mostly-idle processes costs one small frame each, and a continuation is one serializable object rather than a walked stack.
- **Region-oriented memory lifecycle.** Memory is a structured ownership graph, not a flat heap. Reference validation is constant-time and consults no global allocator state, and teardown frees an entire address space in a single bounded pass.
- **Temporal integrity via epochs.** Spatial isolation answers *who can talk to whom*; epochs add *when, and in what generation*. A delayed message from a defunct generation can never reach its successor, even under a shared PID.
- **Auditable by design.** Every state transition is an explicit, typed, capability-bearing interaction that can be logged, replayed, and model-checked.

## Where It Fits

HALOS targets environments where uptime is critical and iteration speed matters:

- Embedded and robotics systems
- Industrial control and long-lived devices
- Network infrastructure and edge compute
- Research platforms and OS experimentation

HALOS is not a general-purpose OS. It is a runtime for resilient systems where components can fail, restart, and evolve without stopping the machine.

## Non-Goals

POSIX compatibility, a rich syscall surface, and performance optimization as a primary axis are explicitly out of scope. If a request cannot be expressed as a typed, capability-bearing message, it does not belong in the system.

## Status and Forward Targets

HALOS is a **research-grade draft in pre-development.** The architecture and component documentation is at v0.5.0. Full ABI documentation, component specifications, and design principles live in the project wiki.

Planned forward targets include:

- **Full System V AMD64 ABI compatibility.** The per-scheduler-thread running stack is already System V-conformant with an intact red zone, so ABI-conformant precompiled code runs natively today; full ABI compatibility is the target.
- **A stable kernel ABI.** Versioning currently exists to detect incompatibility rather than span it. A frozen, stable ABI is a forward goal.
- **Toolchain.** A build target for rustc that compiles to a halos-native execution format, utilizing the capability of the memory management subsystem, which aligns memory ownership with execution semantics. 
- **An Illumos ELF64 syscall-surface compatibility strategy**, far-forward target, layered above the manifest model without restoring ELF to privileged status.
- Frame capture, serialization, and process migration across scheduler threads, building on the position-independent frame.
- Formal verification: constructor-atomicity, manifest-soundness, capability-flow, and crash-containment proofs.

*Author's note:* Thanks for checking out this project! It is a solo endeavour and very much a work in progress. Development takes place in a private git repository, and changes are published here when they are ready to see the light of day. Up next is the project wiki and documentation (including a process-lifecyle dataflow chart), which is in a semi-stable form pending a v0.5 revision currently in progress.

If you have any questions or comments drop by the discussions page!


