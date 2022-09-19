use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[path = "../src/bin/server.rs"]
pub mod server;
use server::listener;

#[path = "../src/bin/client.rs"]
pub mod client;
use client::{client_check_ipv6_interfaces, new_sender};

/// Our generic test over different IPs
fn test_server_listener(addr: String, logfile: PathBuf) {
    //let addr = SocketAddr::new(addr, port);

    //let client_done = Arc::new(AtomicBool::new(false));
    //let _notify = NotifyServer(Arc::clone(&client_done));

    // start server
    listener(addr.clone(), logfile);

    sleep(Duration::from_millis(10));
    let message = b"Hello from client!";
    let addr: SocketAddr = addr.parse().expect("parsing socket address");

    if addr.is_ipv4() {
        let socket = new_sender(&addr).expect("could not create ipv4 sender!");
        socket
            .send_to(message, &addr)
            .expect("could not send to socket!");
    } else {
        let socket = client_check_ipv6_interfaces(&addr).expect("could not create ipv6 sender!");
        socket
            .send_to(message, &addr)
            .expect("could not send to socket!");
    }
}

#[test]
fn test_server_ipv4_unicast() {
    let ipv4 = "127.0.0.1:9900".to_string();
    let logfile: PathBuf = PathBuf::from_str("../testdata/streamoutput_ipv4_unicast.log").unwrap();
    test_server_listener(ipv4, logfile);
}

#[test]
fn test_server_ipv4_multicast() {
    let ipv4 = "221.0.0.110:9901".to_string();
    let logfile: PathBuf =
        PathBuf::from_str("../testdata/streamoutput_ipv4_multicast.log").unwrap();
    test_server_listener(ipv4, logfile);
}

#[test]
fn test_server_ipv6_unicast() {
    let listen = "[::0]:9902".to_string();
    let logfile: PathBuf = PathBuf::from_str("../testdata/streamoutput_ipv6_unicast.log").unwrap();
    test_server_listener(listen, logfile);
}
