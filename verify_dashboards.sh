#!/bin/bash
set -e

# Kill any existing instance
pkill -f "cz-hub" || true

# Start server in background
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

AUTH_HEADER="Authorization: Bearer $ROOT_KEY"

echo "--- Testing Dashboards ---"
# Create Dashboard
DASH=$(curl -s -X POST -H "$AUTH_HEADER" -H "Content-Type: application/json" -d '{"name":"TestDash","description":"My dashboard"}' http://127.0.0.1:3000/api/dashboards)
echo "Created Dashboard: $DASH"
ID=$(echo $DASH | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

# Get Dashboard
curl -s -H "$AUTH_HEADER" http://127.0.0.1:3000/api/dashboards/$ID | grep "TestDash" > /dev/null
echo "Dashboard $ID retrieved"

# Update Dashboard
UPDATE_PAYLOAD='{"layout":[{"i":"w1","x":0,"y":0,"w":4,"h":2}],"widgets":[{"id":"w1","type":"time_series","title":"CPU","query":"SELECT * FROM cpu"}]}'
curl -s -X PUT -H "$AUTH_HEADER" -H "Content-Type: application/json" -d "$UPDATE_PAYLOAD" http://127.0.0.1:3000/api/dashboards/$ID | grep "w1" > /dev/null
echo "Dashboard $ID updated with widgets"

# List Dashboards
curl -s -H "$AUTH_HEADER" http://127.0.0.1:3000/api/dashboards | grep $ID > /dev/null
echo "Dashboard $ID found in list"

# Delete Dashboard
STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE -H "$AUTH_HEADER" http://127.0.0.1:3000/api/dashboards/$ID)
if [ "$STATUS" == "204" ]; then
    echo "Dashboard $ID deleted"
else
    echo "Failed to delete dashboard $ID (Status: $STATUS)"
    exit 1
fi

# Cleanup
kill $SERVER_PID
echo "Verification Complete"
