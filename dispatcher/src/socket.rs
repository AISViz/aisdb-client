use std::io;
use std::net::SocketAddr;

use socket2::{Domain, Protocol, Socket, Type};

/// On unix bind to the multicast address
#[cfg(unix)]
pub fn bind_socket(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms737550(v=vs.85).aspx
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
    socket.set_read_timeout(None)?;
    //#[cfg(debug_assertions)]
    //use std::time::Duration;
    //#[cfg(debug_assertions)]
    //socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    Ok(socket)
}
