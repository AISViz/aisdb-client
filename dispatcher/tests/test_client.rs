use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::thread::JoinHandle;
use std::time::Duration;

#[path = "../src/bin/client.rs"]
mod client;
use client::client_socket_stream;

#[path = "../src/bin/server.rs"]
mod server;
use server::listener;

fn new_server(listen_addr: IpAddr, port: u16, path: PathBuf, multicast: bool) -> JoinHandle<()> {
    let socketaddr = SocketAddr::new(listen_addr, port);
    //let downstream_done = Arc::new(AtomicBool::new(false));
    //let _notify = NotifyServer(Arc::clone(&downstream_done));
    listener(
        socketaddr.to_string(),
        socketaddr,
        path,
        multicast,
        //downstream_done,
    )
}

fn new_client(file: File, target_addr: IpAddr, port: u16) -> Result<UdpSocket, std::io::Error> {
    let targetsocketaddr = SocketAddr::new(target_addr, port);
    let reader = BufReader::new(file);
    client_socket_stream(reader, targetsocketaddr)
    //client_socket_stream(file, targetsocketaddr)
}

fn truncate(path: PathBuf) -> Result<String, std::io::Error> {
    let info = match File::open(&path) {
        Ok(f) => {
            format!("{}", f.metadata().unwrap().len())
        }
        Err(e) => {
            eprintln!("{}", e);
            format!("0")
        }
    };

    File::create(&path)?;
    Ok(info)
}

fn test_client(
    pathstr: &str,
    listen_addr: IpAddr,
    target_addr: IpAddr,
    port: u16,
    multicast: bool,
) {
    let bytesize = truncate(PathBuf::from_str(pathstr).unwrap());
    let bytesize = match bytesize {
        Ok(b) => b,
        Err(e) => panic!("could not fin d{}: {}", pathstr, e),
    };
    new_server(
        listen_addr,
        port,
        PathBuf::from_str(pathstr).unwrap(),
        multicast,
    );
    sleep(Duration::from_millis(20));

    let file = File::open(TESTDATA).expect("opening test data");
    let _ = new_client(file, target_addr, port);
    println!("log size: {}", bytesize);
}

//const TESTDATA: &str = "./tests/test_data_20211101.nm4";
const TESTDATA: &str = "./tests/test_data_random.bin";
const TESTINGDIR: &str = "./tests/";

#[test]
fn test_client_socket_stream_unicast_ipv4() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv4_unicast.log"].join(&"");
    let listen_addr: IpAddr = Ipv4Addr::new(0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    let target_addr: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
    test_client(pathstr, listen_addr, target_addr, 9910, false)
}

#[test]
fn test_client_socket_stream_multicast_ipv4() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv4_multicast.log"].join(&"");
    let target_addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    let listen_addr = target_addr.clone();
    //assert!(listen_addr.is_multicast());
    assert!(target_addr.is_multicast());
    test_client(pathstr, listen_addr, target_addr, 9911, true)
}

#[test]
fn test_client_socket_stream_unicast_ipv6() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv6_unicast.log"].join(&"");
    let listen_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into();
    assert!(!listen_addr.is_multicast());
    let target_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1).into();
    test_client(pathstr, listen_addr, target_addr, 9912, false)
}

// TODO: fix multicast over ipv6
// this test hangs instead of throwing an error
#[test]
fn test_client_socket_stream_multicast_ipv6() {
    let pathstr = &[TESTINGDIR, "streamoutput_client_ipv6_multicast.log"].join(&"");
    //let listen_addr: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into();
    let addr: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    assert!(addr.is_multicast());

    test_client(pathstr, addr, addr, 9913, true)
}
