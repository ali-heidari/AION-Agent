# AION-Agent

## usage
```
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative

CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative --config 'target."cfg(all())".runner="sudo -E"'


sudo docker build -t aion-agent .

sudo docker run -it  --name aaa --cap-add=NET_ADMIN --cap-add=SYS_ADMIN --memory=50M --cpus=1 aion-agent -- busy

monitor dockers network
sudo docker run -it --rm   --net=container:aaa   nicolaka/netshoot tcpdump -i eth0 -nn -e

Add network
sudo docker network create -d macvlan   --subnet=192.168.1.0/24   --gateway=192.168.1.1   -o parent=wlp3s0   mymacvlan

access from host
sudo ip link add link wlp3s0 name host_link type macvlan mode bridge
sudo ip addr add 192.168.1.254/24 dev host_link
sudo ip link set dev host_link up

run container with macvlan network
 sudo docker run -it  --name lll --cap-add=NET_ADMIN --cap-add=SYS_ADMIN --memory=50M --cpus=1  --network=mymacvlan --ip=192.168.1.70 aion-agent 
```