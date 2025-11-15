#!/bin/bash

# Start the aion-agent in the background
/usr/bin/aion-agent &

# Start the Python server in the foreground
# This must be the final command and run in the foreground (no &)
python3 -m http.server 8123 --bind 0.0.0.0
