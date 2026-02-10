# lacrimosa

`lacrimosa` is a local-first control center for the `cz` sequencer.

## One-command launch

```bash
lacrimosa
```

This launches:
- the sequencer (`cz start`)
- the web control center on `http://127.0.0.1:3000`

On startup, copy the generated root API key from logs and paste it into the UI.

## Manual launch

```bash
cd crates/cz-hub/ui
npm install
npm run build

cd /home/x123la/repos/New_Project
cargo run -p cz-hub
```

In another terminal:

```bash
cargo run -p cz-cli -- start --journal journal.db
```

## Project layout

- `crates/cz-core`: core event model and ordering
- `crates/cz-io`: sequencer engine and journal I/O
- `crates/cz-hub`: backend for APIs/websocket + static UI hosting
- `crates/cz-hub/ui`: React/Vite frontend
- `crates/cz-cli`: CLI for runtime operations
