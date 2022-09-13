use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

#[path = "../socket.rs"]
pub mod socket;
use socket::{new_sender, new_sender_ipv6};

fn client_socket_stream(mut reader: BufReader<File>, addr: SocketAddr) {
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

    let mut buf = vec![];
    while let Ok(_) = reader.read_until(b'\n', &mut buf) {
        //let mut buf = vec![0u8; 32];
        //while let Ok(_) = reader.read_exact(&mut buf) {
        if buf.is_empty() {
            break;
        }
        server_socket
            .send_to(&buf, &addr)
            .expect("could not send message to server socket!");
        buf = vec![];
    }
}

fn main() {
    // read antenna socket data
    // downsampling
    // send to server

    // todo: read from args
    const PORT: u16 = 9923;
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 123).into();
    assert!(addr.is_multicast());
    let socketaddr = SocketAddr::new(addr, PORT);

    // todo: autoconfigure device path
    let devpath = "/dev/random";

    let file = File::open(devpath).expect(format!("opening {}", devpath).as_str());
    let reader = BufReader::new(file);
    client_socket_stream(reader, socketaddr)
}

#[cfg(test)]
pub const PORT: u16 = 9922;
#[cfg(test)]
#[path = "server.rs"]
#[cfg(test)]
mod server;
#[cfg(test)]
use server::{multicast_listener, NotifyServer};

#[test]
fn test_client_socket_stream() {
    pub use std::net::{Ipv4Addr, Ipv6Addr};
    pub use std::sync::atomic::AtomicBool;
    pub use std::sync::Arc;

    // start server
    //let addr = *IPV4;
    let addr: IpAddr = Ipv4Addr::new(224, 0, 0, 123).into();
    //let addr: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
    assert!(addr.is_multicast());
    let socketaddr = SocketAddr::new(addr, PORT);
    let client_done = Arc::new(AtomicBool::new(false));
    let _notify = NotifyServer(Arc::clone(&client_done));
    multicast_listener("0", client_done, socketaddr);

    // stream some random bytes to the server
    let file = File::open("/dev/random").expect("opening random!");
    let reader = BufReader::new(file);
    let _ = client_socket_stream(reader, socketaddr);
}
