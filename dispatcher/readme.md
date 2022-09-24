# Network Dispatcher
Client/proxy/server networked socket dispatcher. Streams files and raw socket 
data over the network.

### Features
- [X] Stream arbitrary data over the network
- [X] Complete UDP networking stack
  - Send, proxy, reverse-proxy, and receive to/from multiple endpoints simultaneously
  - Stream multiplexing and aggregation
  - [Multicast](https://en.wikipedia.org/wiki/Multicast) reverse-proxy IP routing
- [X] Fast
  - 500+ Mbps read/transfer/write speed via UDP
- [X] Minimal 
  - Compiled sizes < 350Kb
  - Tiny memory footprint
  - No shared resources, 1 thread per input socket

### Compatible with
- [X] UDP
- [ ] TCP (Partial support / planned feature)
- [X] IPv4
- [X] IPv6
- [X] Unix
- [X] Windows




### Client

Stream data from the client to logging servers. The `--server_addr` option may 
be repeated for multiple server hosts.
The `--tee`/`-t` flag may be used to copy input to stdout

```
cargo run --bin client -- --path '/dev/random' --server_addr '127.0.0.1:9921'
```

### Proxy

Forward UDP packets from upstream addresses to downstream addresses. 
Options `--listen_addr` and `--downstream_addr` may be repeated for multiple 
endpoints. The `--tee`/`-t` flag may be used to copy input to stdout

```
cargo run --bin proxy -- --listen_addr '0.0.0.0:9921' --downstream_addr '[::1]:9922' --tee 
```

### Reverse-Proxy

Forward UDP packets from upstream to new incoming TCP client connections.
UDP packets will be routed via the multicast channel to listeners on each TCP 
client handler thread

```
cargo run --bin reverse_proxy -- --udp_listen_addr '0.0.0.0:9921' --tcp_listen_addr '0.0.0.0:9921' --multicast_addr '224.0.0.1:9922'
```

### Server

Start the logging server. The `--listen_addr` option may be repeated to listen 
for incoming messages from multiple sockets

```
cargo run --bin server -- --path logfile.log --listen_addr '0.0.0.0:9921' --listen_addr '[::]:9922'
```


Use `--help`/`-h` to view help messages


## Alternatives

This utility is intended to be lightweight, fast, and secure, while maintaining stream aggregation and multiplexing.

- For single point-to-point streaming with a more complete feature set, see [Netcat](https://en.wikipedia.org/wiki/Netcat)
- For a feature-rich proxy server with static file serving, file caching, and load balancing, see [Nginx](https://en.wikipedia.org/wiki/Nginx)
