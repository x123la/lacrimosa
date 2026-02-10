# Decision Log: LACRIMOSA Control Center

This document tracks the evolution of the Control Center design, alternatives considered, and the rationale for final decisions.

## 🏁 1. Intent & Context
- **Project**: LACRIMOSA (High-performance sequencer)
- **Goal**: Build a "Control Center" for data interaction, visualization, and external integration.
- **Scope**: Hybrid (Analysis, Observability, Connectivity, Interaction).
- **Core Requirement**: Acts as an "accessory" to the `cz` sequencer.
- **Architecture**: Remote Hub / Plugin Architecture (selected for maximum flexibility).

## 🧠 2. Understanding Lock
- **Verdict**: Remote Hub (Option B) implemented via a local web server (`cz-hub`).
- **Tech Stack**: Rust (Back) + React/Vite (Front).
- **Communication**: WebSockets for low-latency sampling.

## 🛠️ 3. Design Decisions
*Pending decisions...*
