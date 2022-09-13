use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[path = "../src/bin/server.rs"]
pub mod server;
use server::{multicast_listener, NotifyServer};

#[path = "../src/bin/client.rs"]
pub mod client;
use client::{new_sender, new_sender_ipv6};

pub const PORT: u16 = 9923;

/// Our generic test over different IPs
fn test_multicast(test: &'static str, addr: IpAddr) {
    //#[path = "client.rs"]
    //pub mod client;

    assert!(addr.is_multicast());
    let addr = SocketAddr::new(addr, PORT);

    let client_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&client_done));

    // start server
    multicast_listener(test, client_done, addr);

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
fn test_server_ipv4_multicast() {
    let ipv4: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    test_multicast("ipv4", ipv4);
}

/*
#[test]
fn test_server_ipv6_multicast() {
    let ipv6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    test_multicast("ipv6", ipv6);
}
*/
