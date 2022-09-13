use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[path = "../src/bin/client.rs"]
mod client;
use client::client_socket_stream;

#[path = "../src/bin/server.rs"]
mod server;
use server::{multicast_listener, NotifyServer};

pub const PORT: u16 = 9922;

#[test]
fn test_client_socket_stream() {
    // start server
    //let addr = *IPV4;
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    //let addr: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    assert!(addr.is_multicast());
    let socketaddr = SocketAddr::new(addr, PORT);
    let downstream_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&downstream_done));
    multicast_listener("0", downstream_done, socketaddr);

    // stream some random bytes to the server
    let file = File::open("/dev/random").expect("opening random!");
    let reader = BufReader::new(file);
    let _ = client_socket_stream(reader, socketaddr);
}
