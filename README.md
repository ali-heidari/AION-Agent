# AION-Agent

## usage
```
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative

CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative --config 'target."cfg(all())".runner="sudo -E"'


sudo docker build -t aion-agent .

sudo docker run --rm -it --name aaa --cap-add=NET_ADMIN --cap-add=SYS_ADMIN aion-agent which libebpf

```