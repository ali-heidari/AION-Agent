#!/bin/bash

# Start the aion-agent in the background
/usr/local/bin/aion-agent -- $2 &


# parameters to know if we need to start mock.py
# if [ "$1" == "busy" ]; then
#     echo "Starting mock agent..."
#     python3 /usr/local/bin/mock.py &
# fi

# Start the Python server in the foreground
# This must be the final command and run in the foreground (no &)
python3 -m http.server 8123 --bind 0.0.0.0
