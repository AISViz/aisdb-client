// https://bluejekyll.github.io/blog/posts/multicasting-in-rust/

extern crate socket2;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_socket, new_socket};

/// server: client socket handler
/// binds a new socket connection on the network multicast channel
fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", addr.ip());
    assert!(addr.ip().is_multicast());

    let socket = new_socket(&addr)?;
    match addr.ip() {
        IpAddr::V4(ref mdns_v4) => {
            // join to the multicast address, with all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref mdns_v6) => {
            // join to the multicast address, with all interfaces (ipv6 uses indexes not addresses)
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_only_v6(true)?;
        }
    };
    if addr.ip().is_ipv6() {
        let mut ipv6_interface: u32 = 0;
        loop {
            // IPv6 requires explicitly defining the host socket interface
            // this varies between hosts, and don't know how to check,
            // so try them all until one works
            //let result = socket.set_multicast_if_v6(ipv6_interface);
            #[cfg(debug_assertions)]
            if ipv6_interface > 32 {
                panic!("no suitable devices!");
            }
            println!(
                "bind_socket: attempting to bind {} on ipv6 interface {}",
                addr, ipv6_interface
            );
            socket.set_multicast_if_v6(ipv6_interface)?;
            match bind_socket(&socket, &addr) {
                Err(e) => match e.raw_os_error() {
                    Some(0) => {
                        ipv6_interface += 1;
                    }
                    _ => {
                        panic!("{}", e);
                    }
                },
                Ok(_) => {
                    break;
                    //return Ok(socket.into());
                }
            }
        }
    }
    //if let Err(e) = bind_socket(&socket, &addr) {
    //    panic!("failed to bind socket!\t{:?}", e)
    // }
    Ok(socket.into())
}

pub fn join_unicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(&addr)?;
    socket.bind(&socket2::SockAddr::from(addr))?;
    Ok(socket.into())
}

/// server socket listener
pub fn listener(
    response: &'static str,
    downstream_done: Arc<AtomicBool>,
    addr: SocketAddr,
    multicast: bool,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let upstream_barrier = Arc::new(Barrier::new(2));
    let downstream_barrier = Arc::clone(&upstream_barrier);

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            let listener = match multicast {
                false => join_unicast(addr).expect("failed to create socket listener!"),
                true => {match join_multicast(addr) {
                    Ok(s) => s,
                    Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
                }},
            };

            #[cfg(debug_assertions)]
            println!("{}:server: joined: {}", response, addr);

            #[cfg(test)]
            upstream_barrier.wait();

            #[cfg(debug_assertions)]
            println!("{}:server: is ready", response);

            #[cfg(debug_assertions)]
            println!(
                "{}:server: client complete {}",
                response,
                downstream_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            // loop until the client indicates it is done
            while !downstream_done.load(std::sync::atomic::Ordering::Relaxed) {
                let mut buf = [0u8; 1024]; // receive buffer

                match listener.recv_from(&mut buf) {
                    Ok((_len, _remote_addr)) => {
                        #[cfg(debug_assertions)]
                        let data = &buf[.._len];

                        #[cfg(debug_assertions)]
                        println!(
                            "{}:server: got data: {} {} from: {}",
                            response,
                            _len,
                            String::from_utf8_lossy(data),
                            _remote_addr
                        );

                        /*
                        // create a socket to send the response
                        let responder =
                        new_socket(&remote_addr).expect("failed to create responder");

                        let remote_socket = SockAddr::from(remote_addr);

                        // we send the response that was set at the method beginning
                        responder
                        .send_to(response.as_bytes(), &remote_socket)
                        .expect("failed to respond");
                        */

                        #[cfg(debug_assertions)]
                        println!(
                            "{}:server: sent response {} to: {}",
                            response, response, _remote_addr
                        );
                    }
                    Err(err) => {
                        println!("{}:server: got an error: {}", response, err);
                    }
                }
            }

            #[cfg(debug_assertions)]
            println!(
                "{}:server: client complete {}",
                response,
                downstream_done.load(std::sync::atomic::Ordering::Relaxed)
            );

            println!("{}:server: client is done", response);
        })
        .unwrap();

    downstream_barrier.wait();
    join_handle
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
    //let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    //assert!(addr.is_multicast());

    let addr: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

    let socketaddr = SocketAddr::new(addr, PORT);
    //pub static IPV4: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 110).into();
    //pub static IPV6: Ipv6Addr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110).into();

    // start client listener
    let downstream_done = Arc::new(AtomicBool::new(false));
    //let _notify = NotifyServer(Arc::clone(&downstream_done));
    //multicast_listener("0", downstream_done, socketaddr);
    listener("0", downstream_done, socketaddr, true);
}
