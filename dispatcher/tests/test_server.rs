use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[path = "../src/bin/server.rs"]
pub mod server;
use server::{listener, NotifyServer};

#[path = "../src/bin/client.rs"]
pub mod client;
use client::{new_sender, new_sender_ipv6};

/// Our generic test over different IPs
fn test_server_listener(test: &'static str, addr: IpAddr, port: u16) {
    let multicast = addr.is_multicast();
    assert!(addr.is_multicast() == multicast);
    let addr = SocketAddr::new(addr, port);

    let client_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&client_done));

    // start server
    listener(test, client_done, addr, multicast);

    // client test code send and receive code after here
    println!("{}:client: running", test);

    let message = b"Hello from client!";

    if addr.is_ipv4() {
        let socket = new_sender(&addr).expect("could not create ipv4 sender!");
        socket
            .send_to(message, &addr)
            .expect("could not send to socket!");
    } else {
        let socket = new_sender_ipv6(&addr).expect("could not create ipv6 sender!");
        socket
            .send_to(message, &addr)
            .expect("could not send to socket!");
    }
}

#[test]
fn test_server_ipv4_unicast() {
    let ipv4: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
    assert!(ipv4.is_ipv4() && !ipv4.is_multicast());
    test_server_listener("ipv4", ipv4, 9900);
}

#[test]
fn test_server_ipv4_multicast() {
    let ipv4: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    assert!(ipv4.is_ipv4() && ipv4.is_multicast());
    test_server_listener("ipv4", ipv4, 9901);
}

#[test]
fn test_server_ipv6_unicast() {
    let listen: IpAddr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into();
    assert!(listen.is_ipv6() && !listen.is_multicast());
    test_server_listener("ipv6", listen, 9902);
}

/*
#[test]
fn test_server_ipv6_multicast() {
    let ipv6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    assert!(ipv6.is_ipv6() && ipv6.is_multicast());
    test_server_listener("ipv6", ipv6, 9902);
}
*/
