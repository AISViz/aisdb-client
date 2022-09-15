use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_socket, new_socket};

/// new upstream socket
/// socket will allow any downstream IP i.e. 0.0.0.0
pub fn new_sender(addr: &SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(addr)?;

    if !addr.is_ipv4() {
        panic!("invalid socket address type!")
    }
    if addr.ip().is_multicast() {
        socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;
    }
    //socket.bind(&SockAddr::from(SocketAddr::new( Ipv4Addr::new(0, 0, 0, 0).into(), 0,)))?;
    let target_addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);
    bind_socket(&socket, &target_addr)?;

    //Ok(socket.into_udp_socket())
    Ok(socket.into())
}

/// new data output socket to the client IPv6 address
/// socket will allow any downstream IP i.e. ::0
pub fn new_sender_ipv6(addr: &SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(addr)?;

    if !addr.is_ipv6() {
        panic!("invalid socket address type!")
    }
    /*
       if addr.ip().is_multicast() && addr.ip().is_ipv6() {
       let mut ipv6_interface: u32 = 0;
       loop {
    // IPv6 requires explicitly defining the host socket interface
    // this varies between hosts, and don't know how to check,
    // so try them all until one works
    let result = socket.set_multicast_if_v6(ipv6_interface);
    #[cfg(debug_assertions)]
    println!(
    "client: attempting to open listening socket on ipv6 interface {}",
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
    }
    */
    //socket.bind(&SockAddr::from(SocketAddr::new( Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0,)))?;
    let target_addr = SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0);
    bind_socket(&socket, &target_addr)?;

    Ok(socket.into())
}

pub fn client_socket_stream(
    mut reader: BufReader<File>,
    addr: SocketAddr,
) -> io::Result<UdpSocket> {
    let server_socket = match addr.is_ipv4() {
        true => new_sender(&addr).expect("could not create ipv4 sender!"),
        false => new_sender_ipv6(&addr).expect("could not create ipv6 sender!"),
    };

    #[cfg(debug_assertions)]
    println!("opening file...");

    let mut buf = vec![];
    while let Ok(_len) = reader.read_until(b'\n', &mut buf) {
        if buf.is_empty() {
            break;
        }

        let msg = &buf;

        #[cfg(debug_assertions)]
        println!("len: {:?}\tmsg: {:?}", _len, String::from_utf8_lossy(msg));

        server_socket
            .send_to(msg, &addr)
            .expect("could not send message to server socket!");
        buf = vec![];
    }
    Ok(server_socket)
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

struct ClientArgs {
    path: std::path::PathBuf,
    //listenaddr: String,
    port: u16,
}

fn parse_args() -> Result<ClientArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    /*
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
    print!("{}", HELP);
    std::process::exit(0);
    }
    */

    let args = ClientArgs {
        port: pargs.value_from_str("--port")?,
        path: pargs.value_from_os_str("--path", parse_path)?,
        //listenaddr: pargs .opt_value_from_str("--port")? .unwrap_or("0.0.0.0".to_string()),
    };
    Ok(args)
}

pub fn main() {
    // read antenna socket data
    // downsampling
    // send to server

    // todo: read from args
    //const PORT: u16 = 9923;
    //let devpath = "../aisdb/tests/test_data_20211101.nm4";
    //let listensocketaddr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), PORT);

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    let listensocketaddr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), args.port);

    let file = File::open(&args.path).unwrap_or_else(|_| panic!("opening {:?}", &args.path));
    let reader = BufReader::new(file);
    let _ = client_socket_stream(reader, listensocketaddr);
}
