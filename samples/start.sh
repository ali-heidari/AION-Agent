#!/bin/bash

/usr/local/bin/aion-agent -- $2 &

python3 -m http.server 8123 --bind 0.0.0.0
