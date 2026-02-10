#!/bin/bash
set -e

# Kill any existing instance
pkill -f "cz-hub" || true

# Start server in background
export RUST_LOG=info
./target/debug/cz-hub --journals journal.db --bind '127.0.0.1:3000' > server.log 2>&1 &
SERVER_PID=$!
sleep 5

# Capture Root Key
ROOT_KEY=$(grep "GENERATED ROOT API KEY" server.log | sed 's/\x1b\[[0-9;]*m//g' | sed 's/.*KEY: //g' | tr -d '\r' | awk '{print $1}')
echo "Captured Root Key: '$ROOT_KEY'"

if [ -z "$ROOT_KEY" ] || [ ${#ROOT_KEY} -lt 10 ]; then
    echo "Failed to capture valid root key"
    grep "GENERATED ROOT API KEY" server.log
    kill $SERVER_PID
    exit 1
fi

export CZ_API_KEY=$ROOT_KEY
CZ_BIN="./target/debug/cz"

echo "--- Testing CLI ---"

echo "1. Connectors List"
$CZ_BIN connectors list

echo "2. Query"
$CZ_BIN query "node_id > 0"

echo "3. Incidents"
$CZ_BIN incidents

echo "4. Traces"
$CZ_BIN traces --limit 5

# Tail is infinite, so we run it with timeout
echo "5. Tail (5 seconds)"
timeout 5s $CZ_BIN tail journal || true

echo "--- Testing Connector Add/Remove ---"
$CZ_BIN connectors add webhook --config '{}'
# We logic in CLI prints "Connector created: 200 OK" but doesn't return ID easily to script.
# We'll list again to see if it's there.
$CZ_BIN connectors list

# Cleanup
kill $SERVER_PID
echo "Verification Complete"
