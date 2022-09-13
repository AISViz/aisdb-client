use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

#[cfg(windows)]
pub fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    let addr = match *addr {
        SocketAddr::V4(addr) => SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port()),
        SocketAddr::V6(addr) => {
            SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port())
        }
    };
    socket.bind(&socket2::SockAddr::from(addr))
}

/// On unixes we bind to the multicast address, which causes multicast packets to be filtered
#[cfg(unix)]
pub fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

// this will be common for all our sockets
pub fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else if addr.is_ipv6() {
        Domain::IPV6
    } else {
        Domain::UNIX
    };

    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_port(true)?;
    socket.set_freebind(true)?;

    // we're going to use read timeouts so that we don't hang waiting for packets
    //socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}

/// new data output socket to the server IPv4 address
/// socket will accept response from any server i.e. 0.0.0.0
pub fn new_sender(addr: &SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(addr)?;

    if !addr.is_ipv4() {
        panic!("invalid socket address type!")
    }
    socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;
    //socket.bind(&SockAddr::from(SocketAddr::new( Ipv4Addr::new(0, 0, 0, 0).into(), 0,)))?;
    let target_addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);
    bind_multicast(&socket, &target_addr)?;

    //Ok(socket.into_udp_socket())
    Ok(socket.into())
}

/// new data output socket to the client IPv6 address
/// socket will accept input from any client i.e. ::0
pub fn new_sender_ipv6(addr: &SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(addr)?;

    let mut ipv6_interface: u32 = 0;

    if !addr.is_ipv6() {
        panic!("invalid socket address type!")
    }
    loop {
        // IPv6 requires explicitly defining the host socket interface
        // this varies between hosts, and don't know how to check,
        // so try them all until one works
        let result = socket.set_multicast_if_v6(ipv6_interface);
        #[cfg(debug_assertions)]
        println!(
            "attempting to open listening socket on ipv6 interface {}",
            ipv6_interface
        );
        match result {
            Err(e) => match e.raw_os_error() {
                Some(99) | Some(101) => {
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
    //socket.bind(&SockAddr::from(SocketAddr::new( Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0,)))?;
    let target_addr = SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0);
    bind_multicast(&socket, &target_addr)?;

    //Ok(socket.into_udp_socket())
    Ok(socket.into())
}
