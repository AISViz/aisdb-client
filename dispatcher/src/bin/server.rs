// https://bluejekyll.github.io/blog/posts/multicasting-in-rust/

//#[macro_use]
//extern crate lazy_static;
extern crate socket2;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;

//use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use socket2::SockAddr;

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_multicast, new_socket};

/*
lazy_static! {
pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
pub static ref IPV6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
}
*/
//pub static IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 110).into();
//pub static IPV6: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();

/// server socket listener
pub fn multicast_listener(
    response: &'static str,
    client_done: Arc<AtomicBool>,
    addr: SocketAddr,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let server_barrier = Arc::new(Barrier::new(2));
    let client_barrier = Arc::clone(&server_barrier);

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            let listener = join_multicast(addr)
                .expect(format!("failed to create listener on address {}", addr).as_str());
            #[cfg(debug_assertions)]
            println!("{}:server: joined: {}", response, addr);

            //server_barrier.wait();

            #[cfg(debug_assertions)]
            println!("{}:server: is ready", response);

            #[cfg(debug_assertions)]
            println!(
                "{}:server: client complete {}",
                response,
                client_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            // loop until the client indicates it is done
            while !client_done.load(std::sync::atomic::Ordering::Relaxed) {
                // test receive and response code will go here...
                let mut buf = [0u8; 64]; // receive buffer

                // we're assuming failures were timeouts, the client_done loop will stop us
                match listener.recv_from(&mut buf) {
                    Ok((_len, remote_addr)) => {
                        //#[cfg(debug_assertions)]
                        let data = &buf[.._len];

                        #[cfg(debug_assertions)]
                        println!(
                            "{}:server: got data: {} from: {}",
                            response,
                            String::from_utf8_lossy(data),
                            remote_addr
                        );

                        // create a socket to send the response
                        let responder =
                            new_socket(&remote_addr).expect("failed to create responder");

                        let remote_socket = SockAddr::from(remote_addr);

                        // we send the response that was set at the method beginning
                        responder
                            //.send_to(response.as_bytes(), &remote_addr)
                            .send_to(response.as_bytes(), &remote_socket)
                            .expect("failed to respond");

                        #[cfg(debug_assertions)]
                        println!(
                            "{}:server: sent response {} to: {}",
                            response, response, remote_addr
                        );
                    }
                    Err(err) => {
                        //println!("{}:server: got an error: {}", response, err);
                        panic!("{}:server: got an error: {}", response, err);
                    }
                }
            }
            #[cfg(debug_assertions)]
            println!(
                "{}:server: client complete {}",
                response,
                client_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            println!("{}:server: client is done", response);
        })
        .unwrap();

    client_barrier.wait();
    join_handle
}

/// server: client socket handler
/// binds a new socket connection on the network multicast channel
fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let ip_addr = addr.ip();

    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", ip_addr);

    let socket = new_socket(&addr)?;

    // depending on the IP protocol we have slightly different work
    match ip_addr {
        IpAddr::V4(ref mdns_v4) => {
            // join to the multicast address, with all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref mdns_v6) => {
            // join to the multicast address, with all interfaces (ipv6 uses indexes not addresses)
            //socket.join_multicast_v6(mdns_v6, ipv6_interface)?;
            match socket.join_multicast_v6(mdns_v6, 0) {
                Err(e) => panic!("{}", e),
                Ok(_) => {}
            }
            match socket.set_only_v6(true) {
                Err(e) => panic!("{}", e),
                Ok(_) => {}
            }
        }
    };

    // bind us to the socket address.
    //socket.bind(&addr)?;
    //Ok(socket.into_udp_socket())
    //match socket.bind(&SockAddr::from(addr)) {
    //match socket.bind(&SockAddr::from(addr)) {
    //    Err(e) => panic!("{}", e),
    //    Ok(_) => {}
    //}
    //?;
    bind_multicast(&socket, &addr)?;
    Ok(socket.into())
}

/// ensure the server is stopped
pub struct NotifyServer(pub Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

pub fn main() {
    // todo: read args from command line
    const PORT: u16 = 9923;
    //let addr = *IPV4;

    // CIDR group 224 => multicast address range
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    //pub static IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 110).into();
    //pub static IPV6: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();

    // start server listener
    let client_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&client_done));
    assert!(addr.is_multicast());
    let socketaddr = SocketAddr::new(addr, PORT);
    let response = "0";
    multicast_listener(response, client_done, socketaddr);
}

#[cfg(test)]
pub const PORT: u16 = 9923;

#[cfg(test)]
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

    use socket::{new_sender, new_sender_ipv6};
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
fn test_ipv4_multicast() {
    let ipv4: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    test_multicast("ipv4", ipv4);
}

/*
#[test]
fn test_ipv6_multicast() {
    let ipv6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();
    test_multicast("ipv6", ipv6);
}
*/
