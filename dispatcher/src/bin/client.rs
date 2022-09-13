use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_multicast, new_socket};

pub fn client_socket_stream(mut reader: BufReader<File>, addr: SocketAddr) {
    let server_socket: UdpSocket;
    match addr.is_ipv4() {
        true => {
            server_socket = new_sender(&addr).expect("could not create ipv4 sender!");
        }
        false => {
            server_socket = new_sender_ipv6(&addr).expect("could not create ipv6 sender!");
        }
    };

    #[cfg(debug_assertions)]
    println!("opening file...");

    //let mut buf = vec![];
    //let mut buf = vec![0u8; 1024];
    let mut buf: Vec<u8>;
    buf = [0u8; 64].to_vec();
    while let Ok(len) = reader.read_until(b'\n', &mut buf) {
        //let mut buf = vec![0u8; 32];
        //while let Ok(_) = reader.read_exact(&mut buf) {
        if buf.is_empty() {
            break;
        }

        let msg = &buf;

        #[cfg(debug_assertions)]
        println!("len: {:?}\tmsg: {:?}", len, String::from_utf8_lossy(msg));

        server_socket
            .send_to(msg, &addr)
            .expect("could not send message to server socket!");
        //buf = vec![];
        //buf = [0u8; 64];
        buf = [0u8; 64].to_vec();

        #[cfg(debug_assertions)]
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
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

fn main() {
    // read antenna socket data
    // downsampling
    // send to server

    // todo: read from args
    const PORT: u16 = 9923;
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 110).into();
    assert!(addr.is_multicast());
    let socketaddr = SocketAddr::new(addr, PORT);

    // todo: autoconfigure device path
    let devpath = "/dev/random";

    let file = File::open(devpath).expect(format!("opening {}", devpath).as_str());
    let reader = BufReader::new(file);
    client_socket_stream(reader, socketaddr)
}
