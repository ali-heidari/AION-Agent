#!/bin/bash
# Injects the fixed mock-for-small-container.py into all running aixker-agent
# containers and starts CPU pressure in the background.
#
# Usage:
#   ./load.sh          — inject and start load on all aixker-agent containers
#   ./load.sh --stop   — stop load on all aixker-agent containers

MOCK_FILE="$(dirname "$0")/../agent/scripts/mock-for-small-container.py"

if [ ! -f "$MOCK_FILE" ]; then
    echo "ERROR: mock-for-small-container.py not found at $MOCK_FILE"
    exit 1
fi

CONTAINERS=$(docker ps --filter "ancestor=aixker-agent" --format "{{.Names}}")

if [ -z "$CONTAINERS" ]; then
    echo "No running aixker-agent containers found."
    exit 1
fi

if [ "$1" == "--stop" ]; then
    for container in $CONTAINERS; do
        echo "Stopping load on: $container"
        docker exec "$container" pkill -f mock.py 2>/dev/null
    done
    echo "Done."
    exit 0
fi

for container in $CONTAINERS[0..8]; do
    echo "Injecting into: $container"
    docker cp "$MOCK_FILE" "$container:/usr/local/bin/mock.py"
    docker restart "$container"  # Restart to ensure the new mock.py is used
    docker exec -d "$container" python3 /usr/local/bin/mock.py
    sleep 5
done

echo ""
echo "Load started on $(echo "$CONTAINERS" | wc -l) container(s)."
echo "Run with --stop to kill it."
