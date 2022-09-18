# Network Dispatcher
Client/server application to send data from a file descriptor or socket over the network to one or more logging servers

### Features
- [X] Stream arbitrary data over the network
- [X] Log data from multiple clients simultaneously
- [X] Stream to multiple logging servers simultaneously 
- [X] Minimal (2 dependencies, compiled size ~300kb)
- [X] Extremely fast

### Compatible with:

- [ ] TCP (Planned feature)
- [X] UDP
- [X] IPv4
- [X] IPv6
- [X] Unix
- [X] Windows
- [X] [Multicast](https://en.wikipedia.org/wiki/Multicast) IP routing




### Server

Start the logging server
```
cargo run --bin server --release -- --port 9920 --listen_addr 0.0.0.0 --path logfile.log
```

### Client

Stream data from the client to the logging server:
```
cargo run --bin client --release -- --port 9920 --path /dev/random --server_addr 127.0.0.1
```

