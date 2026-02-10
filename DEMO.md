# 🚀 LACRIMOSA: System Integration Demo

To see the system in action with live telemetry, follow these steps in separate terminals:

### 1. Start the Hub (Control Center)
```bash
cargo run -p cz-hub
```
*Navigates to: http://localhost:3000*

### 🧬 LACRIMOSA V2 (The Control Center)

LACRIMOSA Control Center is a high-fidelity diagnostic and command suite for the LACRIMOSA sequencer. It is designed for low-latency, zero-copy inspection of real-time event streams.

## 🚀 Current Use Case
- **Real-Time Health**: Monitor TPS, BPS, and Ring Buffer saturation via WebSocket telemetry.
- **Direct Memory Inspection**: Deep-dive into the mmap'd `journal.db` with Hex/ASCII diagnostics.
- **Causal Topology**: Visualize the relationships between Nodes and logical Streams in a force-graph.
- **Formal Verification**: One-click execution of Kani proofs to ensure system-wide safety invariants.
- **Load Simulation**: Inject synthetic traffic to stress-test your sequencer configurations.

## 🛑 What it is NOT (Intentional Scope)
- **Not a Persistent Database**: Metrics are stored in a 1-hour rolling RAM buffer. For long-term analytics, use the Export feature.
- **Not for Remote Cluster Orchestration**: It is optimized for local-first developer productivity and single-instance deep inspection.
- **Not a "Write" App**: Outside of the Simulator, it is an observability-first control plane. It observes and verifies the sequencer without interfering with its critical path.

## 🛠 Power User Tips
- **Ctrl+K**: Open the Command Palette to navigate instantly.
- **Tab Key**: Cycle between pages (Overview -> Explorer -> Topology).
- **Themes**: Switch between OLED Black or Indigo Night for low-light debugging.

### 2. Start the Sequencer (Engine)
In another terminal, initialize and start the sequencer:
```bash
# Create a 1GB test journal
cargo run -p cz-cli -- start --journal journal.db
```

### 3. Send Mock Data (The Pulse)
In a third terminal, send some UDP data to the sequencer:
```bash
# Send a test packet (32-byte header + payload)
echo "LACRIMOSA-DATA-PAYLOAD" | nc -u -w1 127.0.0.1 9000
```

### 4. Observe
Watch the **Events Processed** and **Pulse Chart** update in real-time on the dashboard!
