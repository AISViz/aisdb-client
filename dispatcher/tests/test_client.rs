use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

#[path = "../src/bin/client.rs"]
mod client;
use client::client_socket_stream;

#[path = "../src/bin/server.rs"]
mod server;
use server::{listener, NotifyServer};

pub const PORT: u16 = 9921;

fn new_server(listen_addr: IpAddr, port: u16, multicast: bool) -> JoinHandle<()> {
    let socketaddr = SocketAddr::new(listen_addr, port);
    let downstream_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&downstream_done));
    listener("0", downstream_done, socketaddr, multicast)
}

fn new_client(file: File, target_addr: IpAddr, port: u16) -> Result<UdpSocket, std::io::Error> {
    let targetsocketaddr = SocketAddr::new(target_addr, port);
    let reader = BufReader::new(file);
    client_socket_stream(reader, targetsocketaddr)
}

#[test]
fn test_client_socket_stream_unicast_ipv4() {
    let listen_addr: IpAddr = Ipv4Addr::new(0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    new_server(listen_addr, PORT, false);

    let target_addr: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
    let file = File::open("../aisdb/tests/test_data_20211101.nm4").expect("opening test data");
    let _ = new_client(file, target_addr, PORT);
}

#[test]
fn test_client_socket_stream_multicast_ipv4() {
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    assert!(addr.is_multicast());
    new_server(addr, PORT, true);

    let file = File::open("../aisdb/tests/test_data_20211101.nm4").expect("opening test data");
    let _ = new_client(file, addr, PORT);
}

#[test]
fn test_client_socket_stream_unicast_ipv6() {
    let listen_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    new_server(listen_addr, PORT, false);

    let target_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into();
    let file = File::open("../aisdb/tests/test_data_20211101.nm4").expect("opening test data");
    let _ = new_client(file, target_addr, PORT);
}

// TODO: fix multicast over ipv6
// this test hangs instead of throwing an error
/*
#[test]
fn test_client_socket_stream_multicast_ipv6() {
    let addr: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    assert!(addr.is_multicast());

    new_server(addr, PORT, true);

    let file = File::open("../aisdb/tests/test_data_20211101.nm4").expect("opening test data");
    let _ = new_client(file, addr, PORT);
}
*/
