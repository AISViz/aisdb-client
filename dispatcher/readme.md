# Network Dispatcher
Client/proxy/server networked socket dispatcher. Streams files and raw socket 
data over the network.

### Features
- [X] Stream arbitrary data over the network
- [X] Send, proxy, and receive data from multiple endpoints simultaneously
- [X] Minimal (compiled size < 350kb)
- [X] Fast

### Compatible with
- [ ] TCP (Planned feature)
- [X] UDP
- [X] IPv4
- [X] IPv6
- [X] Unix
- [X] Windows
- [X] [Multicast](https://en.wikipedia.org/wiki/Multicast) IP routing




### Client

Stream data from the client to logging servers. The `--server_addr` option may 
be repeated for multiple server hosts.
The `--tee`/`-t` flag may be used to copy input to stdout

```
cargo run --bin client -- --path '/dev/random' --server_addr '127.0.0.1:9921'
```

### Proxy

Forward UDP packets from upstream addresses to downstream addresses. 
Options `--listen_addr` and `--downstream_addr` may be repeated for multiple endpoints.
The `--tee`/`-t` flag may be passed to copy input to stdout

```
cargo run --bin proxy -- --listen_addr '0.0.0.0:9921' --downstream_addr '[::1]:9922' --tee 
```

### Server

Start the logging server. The `--listen_addr` option may be repeated to listen for 
incoming messages from multiple sockets

```
cargo run --bin server -- --path logfile.log --listen_addr '0.0.0.0:9921' --listen_addr '[::]:9922'
```


Use `--help`/`-h` to view help messages
