# Network Dispatcher
Client/server application to send data from a file descriptor over the network to one or more logging servers

### Features
- [X] Stream arbitrary data over the network
- [X] Log data from multiple clients simultaneously
- [ ] Stream to multiple logging servers simultaneously (Planned feature)
- [X] Minimal (2 dependencies, compiled size ~300kb)
- [X] Fast

### Compatible with
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
cargo run --bin server --release -- --listen_addr 0.0.0.0:9920 --path logfile.log
```

### Client

Stream data from the client to the logging server

```
cargo run --bin client --release -- --path /dev/random --server_addr 127.0.0.1:9920 --server_addr [::1]:9921
```


The `--tee` or `-t` flag may be used by the client to copy file descriptor input to stdout
