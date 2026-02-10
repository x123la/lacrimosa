#!/bin/bash
set -e

# Kill any existing instance
pkill -f "cz-hub" || true

# Start server in background
export RUST_LOG=info
./target/debug/cz-hub --journals journal.db --bind '127.0.0.1:3000' > server.log 2>&1 &
SERVER_PID=$!
sleep 5

# Capture Root Key - aggressive ANSI stripping
# usage of perl for robust ansi stripping if sed fails (sed syntax varies)
# But standard sed for ansi: s/\x1b\[[0-9;]*m//g 
# Note: \x1b might not work in all seds, use \033
ROOT_KEY=$(grep "GENERATED ROOT API KEY" server.log | sed 's/\x1b\[[0-9;]*m//g' | sed 's/.*KEY: //g' | tr -d '\r' | awk '{print $1}')
echo "Captured Root Key: '$ROOT_KEY'"

if [ -z "$ROOT_KEY" ] || [ ${#ROOT_KEY} -lt 10 ]; then
    echo "Failed to capture valid root key"
    grep "GENERATED ROOT API KEY" server.log
    kill $SERVER_PID
    exit 1
fi

echo "--- Testing Auth ---"

# 1. 401 without key
STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3000/api/metrics/history)
if [ "$STATUS" == "401" ]; then
    echo "PASS: 401 without key"
else
    echo "FAIL: Expected 401, got $STATUS"
    kill $SERVER_PID
    exit 1
fi

# 2. 200 with key
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $ROOT_KEY" http://127.0.0.1:3000/api/metrics/history)
if [ "$STATUS" == "200" ]; then
    echo "PASS: 200 with key"
else
    echo "FAIL: Expected 200, got $STATUS"
    kill $SERVER_PID
    exit 1
fi

# 3. Create new key
NEW_KEY_JSON=$(curl -s -X POST -H "Authorization: Bearer $ROOT_KEY" -H "Content-Type: application/json" -d '{"label":"TestKey","scopes":["read"]}' http://127.0.0.1:3000/api/auth/keys)
echo "Create Key Response: $NEW_KEY_JSON"
NEW_KEY=$(echo $NEW_KEY_JSON | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
KEY_ID=$(echo $NEW_KEY_JSON | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Created new key: $KEY_ID"

# 4. Use new key
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $NEW_KEY" http://127.0.0.1:3000/api/metrics/history)
if [ "$STATUS" == "200" ]; then
    echo "PASS: 200 with new key"
else
    echo "FAIL: Expected 200 with new key, got $STATUS"
    kill $SERVER_PID
    exit 1
fi

# 5. Revoke key
curl -s -X DELETE -H "Authorization: Bearer $ROOT_KEY" http://127.0.0.1:3000/api/auth/keys/$KEY_ID > /dev/null
echo "Revoked key $KEY_ID"

# 6. Verify revocation (401)
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $NEW_KEY" http://127.0.0.1:3000/api/metrics/history)
if [ "$STATUS" == "401" ]; then
    echo "PASS: 401 with revoked key"
else
    echo "FAIL: Expected 401 with revoked key, got $STATUS"
    kill $SERVER_PID
    exit 1
fi

# Cleanup
kill $SERVER_PID
echo "Verification Complete"
