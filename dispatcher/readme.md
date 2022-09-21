# Network Dispatcher
Client/proxy/server application to send data from a file, file descriptor 
(unix), or handle (windows) over the network to one or more logging servers

### Features
- [X] Stream arbitrary data over the network
- [X] Log data from multiple clients simultaneously
- [X] Proxy from multiple clients to multiple servers simultaneously
- [X] Stream to multiple logging servers simultaneously
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

Start the logging server. the `--listen_addr` may be repeated to listen for 
incoming messages from multiple sockets
```
cargo run --bin server --release -- --path logfile.log --listen_addr '0.0.0.0:9920' --listen_addr '[::]:9921'
```

### Client

Stream data from the client to logging servers. The `--server_addr` option may 
be repeated for multiple server hosts.
The `--tee` or `-t` flag may be used to copy input to stdout

```
cargo run --bin client --release -- --path /dev/random --server_addr '127.0.0.1:9920' --server_addr '[::1]:9921'
```

### Proxy

Forward UDP packets from upstream addresses to downstream addresses. 
Options `--listen_addr` and `--downstream_addr` may be repeated for multiple endpoints.
The `--tee` or `-t` flag may be passed to copy input to stdout

```
cargo run --bin proxy --release  -- --listen_addr [::]9921 --downstream_addr [::2]9922 --tee 
```

