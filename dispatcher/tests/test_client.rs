use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time::Duration;

#[path = "../src/bin/client.rs"]
mod client;
use client::client_socket_stream;

#[path = "../src/bin/server.rs"]
mod server;
use server::{listener, NotifyServer};

fn new_server(listen_addr: IpAddr, port: u16, path: PathBuf) -> JoinHandle<()> {
    let socketaddr = SocketAddr::new(listen_addr, port);
    let downstream_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&downstream_done));
    listener(socketaddr.to_string(), socketaddr, path, downstream_done)
}

fn new_client(file: File, target_addr: IpAddr, port: u16) -> Result<UdpSocket, std::io::Error> {
    let targetsocketaddr = SocketAddr::new(target_addr, port);
    let reader = BufReader::new(file);
    client_socket_stream(reader, targetsocketaddr)
    //client_socket_stream(file, targetsocketaddr)
}

fn truncate(path: PathBuf) -> Result<(), std::io::Error> {
    println!(
        "truncating log: {} bytes",
        File::open(&path)
            .expect("opening logfile")
            .metadata()
            .unwrap()
            .len()
    );
    File::create(&path)?;
    Ok(())
}

const TESTDATA: &str = "../src/aisdb/tests/test_data_20211101.nm4";

#[test]
fn test_client_socket_stream_unicast_ipv4() {
    let pathstr = "../src/testdata/streamoutput_client_ipv4_unicast.log";
    let listen_addr: IpAddr = Ipv4Addr::new(0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    new_server(listen_addr, 9910, PathBuf::from_str(pathstr).unwrap());

    let target_addr: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
    let file = File::open(TESTDATA).expect("opening test data");
    let _ = new_client(file, target_addr, 9910);
    sleep(Duration::from_millis(10));

    let _ = truncate(PathBuf::from_str(pathstr).unwrap());
}

#[test]
fn test_client_socket_stream_multicast_ipv4() {
    let logfile =
        PathBuf::from_str("../src/testdata/streamoutput_client_ipv4_multicast.log").unwrap();
    let listen_addr: IpAddr = Ipv4Addr::new(0, 0, 0, 0).into();
    new_server(listen_addr, 9911, logfile);

    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    assert!(addr.is_multicast());
    let file = File::open(TESTDATA).expect("opening test data");
    let _ = new_client(file, addr, 9911);
}

#[test]
fn test_client_socket_stream_unicast_ipv6() {
    let logfile =
        PathBuf::from_str("../src/testdata/streamoutput_client_ipv6_unicast.log").unwrap();
    let listen_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    new_server(listen_addr, 9912, logfile);

    let target_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into();
    let file = File::open(TESTDATA).expect("opening test data");
    let _ = new_client(file, target_addr, 9912);
}

// TODO: fix multicast over ipv6
// this test hangs instead of throwing an error
/*
#[test]
fn test_client_socket_stream_multicast_ipv6() {
let addr: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
assert!(addr.is_multicast());

new_server(addr, PORT, true);

let file = File::open("../src/aisdb/tests/test_data_20211101.nm4").expect("opening test data");
let _ = new_client(file, addr, PORT);
}
*/
