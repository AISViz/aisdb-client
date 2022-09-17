use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::str::FromStr;

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
        //socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;
        socket.set_multicast_loop_v4(true)?;
    }
    let target_addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 0);
    bind_socket(&socket, &target_addr)?;

    Ok(socket.into())
}

/// new data output socket to the client IPv6 address
/// socket will allow any downstream IP i.e. ::0
fn new_sender_ipv6(addr: &SocketAddr, ipv6_interface: u32) -> io::Result<UdpSocket> {
    let target_addr = SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port());

    if !addr.is_ipv6() {
        panic!("invalid socket address type!")
    }

    let socket = new_socket(addr)?;
    if addr.ip().is_multicast() {
        let _a = socket.set_multicast_if_v6(ipv6_interface);
        let _b = socket.set_multicast_loop_v6(true);
        socket.set_reuse_address(true)?;
        let _c = bind_socket(&socket, &target_addr);

        assert!(_a.is_ok());
        assert!(_b.is_ok());
        assert!(_c.is_ok());
    } else {
    }
    Ok(socket.into())
    //socket.bind(&SockAddr::from(SocketAddr::new( Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0,)))?;
    //let target_addr = SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), 0);
    //bind_socket(&socket, &target_addr)?;

    //Ok(socket.into())
}

pub fn client_check_ipv6_interfaces(addr: SocketAddr) -> io::Result<UdpSocket> {
    for i in 0..32 {
        #[cfg(debug_assertions)]
        println!("checking interface {}", i);
        let socket = new_sender_ipv6(&addr, i)?;
        let result = socket.send_to(b"", &addr);
        match result {
            Ok(_r) => {
                //
                #[cfg(debug_assertions)]
                println!("opened interface {}:\t{}", i, _r);
                return Ok(socket);
            }
            Err(e) => {
                //#[cfg(debug_assertions)]
                eprintln!("err: could not open interface {}:\t{:?}", i, e)
            }
        }
    }
    panic!("No suitable network interfaces were found!");
}

pub fn client_socket_stream(
    mut reader: BufReader<File>,
    //mut file: File,
    addr: SocketAddr,
) -> io::Result<UdpSocket> {
    let server_socket = match addr.is_ipv4() {
        true => new_sender(&addr).expect("could not create ipv4 sender!"),
        //false => new_sender_ipv6(&addr, client_check_interfaces(addr)).expect("could not create ipv6 sender!"),
        false => client_check_ipv6_interfaces(addr).expect("could not create ipv6 sender!"),
    };

    #[cfg(debug_assertions)]
    println!("opening file...");

    //let mut buf = vec![];
    //while let Ok(_len) = reader.read_until(b'\n', &mut buf) {
    let mut buf = vec![0u8; 1024];
    //while let Ok(_) = reader.read_exact(&mut buf) {
    while let Ok(c) = reader.read(&mut buf) {
        if c == 0 {
            break;
        }

        //#[cfg(debug_assertions)]
        //println!("\n{} client: {:?}", c, String::from_utf8_lossy(&buf[..c]));

        server_socket
            .send_to(&buf[..c], &addr)
            .expect("could not send message to server socket!");
        //buf = vec![];
        buf = vec![0u8; 1024];
    }
    Ok(server_socket)
}

struct ClientArgs {
    listen_addr: String,
    path: std::path::PathBuf,
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
    fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
        Ok(s.into())
    }

    let args = ClientArgs {
        port: pargs.value_from_str("--port")?,
        path: pargs.value_from_os_str("--path", parse_path)?,
        listen_addr: pargs
            .opt_value_from_str("--listen_addr")?
            //.unwrap_or("0.0.0.0".to_string())
            .unwrap_or_else(|| "0.0.0.0".to_string()),
    };

    Ok(args)
}

pub fn main() {
    // read socket or file data
    // TODO: downsampling ?? might require callback or plugin for each data type
    // send to server

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
    let listenaddr = IpAddr::from_str(&args.listen_addr).unwrap();
    let listensocketaddr = SocketAddr::new(listenaddr, args.port);

    let file = File::open(&args.path).unwrap_or_else(|_| panic!("opening {:?}", &args.path));
    let reader = BufReader::new(file);
    let _ = client_socket_stream(reader, listensocketaddr);
    //let _ = client_socket_stream(file, listensocketaddr);
}
