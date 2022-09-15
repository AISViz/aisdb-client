use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

/// On unixes we bind to the multicast address, which causes multicast packets to be filtered
#[cfg(unix)]
pub fn bind_socket(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

/// On Windows, unlike all Unix variants, it is improper to bind to the multicast address
///
/// see https://msdn.microsoft.com/en-us/library/windows/desktop/ms737550(v=vs.85).aspx
#[cfg(windows)]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
#[cfg(windows)]
pub fn bind_socket(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    let addr = match addr.ip().is_multicast() {
        true => match addr {
            SocketAddr::V4(addr) => SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port()),
            SocketAddr::V6(addr) => {
                SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port())
            }
        },
        false => *addr,
    };
    socket.bind(&socket2::SockAddr::from(addr))
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
    //socket.set_freebind(true)?;

    // we're going to use read timeouts so that we don't hang waiting for packets
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    //socket.set_read_timeout(None)?;

    Ok(socket)
}
