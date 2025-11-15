# AION-Agent

## usage
```
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative

CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative --config 'target."cfg(all())".runner="sudo -E"'


sudo docker build -t aion-agent .

sudo docker run -it  --name aaa --cap-add=NET_ADMIN --cap-add=SYS_ADMIN --memory=500M --cpus=1 aion-agent busy

```