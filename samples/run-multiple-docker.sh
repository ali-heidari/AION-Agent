#!/bin/bash

# Starting number
base=60
count=100

for i in $(seq 0 $((count-1))); do
    number=$((base + i))
    echo "Launching container with IP 192.168.1.$number"
    
    sudo docker run -it -d --rm \
        --cap-add=NET_ADMIN \
        --cap-add=SYS_ADMIN \
        --memory=50M \
        --cpus=1 \
        --network=mymacvlan \
        --ip=192.168.1.$number \
        aion-agent
done
1:06