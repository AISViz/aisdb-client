use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{stdout, Result as ioResult};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::process::exit;

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_socket, new_socket};

const HELP: &str = r#"
DISPATCH: CLIENT

USAGE:
  client --path [FILE_DESCRIPTOR] --server_addr [SOCKET_ADDR] ...

  e.g.
  client --path /dev/random --server_addr 127.0.0.1:9920 --server_addr [::1]:9921
  client --path ./readme.md --server_addr 224.0.0.1:9922 --server_addr [ff02::1]:9923 --tee >> logfile.log

FLAGS:
  -h, --help    Prints help information
  -t, --tee     Copy input to stdout

"#;

/// command line arguments
struct ClientArgs {
    path: PathBuf,
    server_addrs: Vec<String>,
    tee: bool,
}

/// retrieve command line arguments as ClientArgs struct
fn parse_args() -> Result<ClientArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
        print!("{}", HELP);
        exit(0);
    }
    let tee = pargs.contains(["-t", "--tee"]);

    fn parse_path(s: &OsStr) -> Result<PathBuf, &'static str> {
        Ok(s.into())
    }

    let args = ClientArgs {
        path: pargs.value_from_os_str("--path", parse_path)?,
        server_addrs: pargs.values_from_str("--server_addr")?,
        tee,
    };

    Ok(args)
}

/// new upstream socket
/// socket will allow any downstream IP i.e. 0.0.0.0
pub fn new_sender(addr: &SocketAddr) -> ioResult<UdpSocket> {
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
fn new_sender_ipv6(addr: &SocketAddr, ipv6_interface: u32) -> ioResult<UdpSocket> {
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
        if _c.is_err() {
            panic!("error binding socket {:?}", _c);
        }
    } else {
    }
    Ok(socket.into())
}

pub fn client_check_ipv6_interfaces(addr: &SocketAddr) -> ioResult<UdpSocket> {
    for i in 0..32 {
        #[cfg(debug_assertions)]
        println!("checking interface {}", i);
        let socket = new_sender_ipv6(addr, i)?;
        let result = socket.send_to(b"", addr);
        match result {
            Ok(_r) => {
                //
                #[cfg(debug_assertions)]
                println!("opened interface {}:\t{}", i, _r);
                return Ok(socket);
            }
            Err(e) => {
                eprintln!("err: could not open interface {}:\t{:?}", i, e)
            }
        }
    }
    panic!("No suitable network interfaces were found!");
}

//pub fn client_socket_stream(path: &PathBuf, server_addr: String, tee: bool) -> ioResult<UdpSocket> {
pub fn client_socket_stream(path: &PathBuf, server_addrs: Vec<String>, tee: bool) -> ioResult<()> {
    //pub fn client_socket_stream(path: &PathBuf, server_addr: String, tee: bool) -> JoinHandle<()> {

    let mut targets = vec![];

    for server_addr in server_addrs {
        let target_addr: SocketAddr = server_addr.parse().expect("parsing server address");
        let target_socket = match target_addr.is_ipv4() {
            true => new_sender(&target_addr).expect("creating ipv4 send socket!"),
            false => {
                client_check_ipv6_interfaces(&target_addr).expect("creating ipv6 send socket!")
            }
        };
        targets.push((target_addr, target_socket));
        println!(
            "logging {}: listening for {}",
            &path.as_os_str().to_str().unwrap(),
            server_addr,
        );
    }
    //#[cfg(debug_assertions)]
    //println!("opening {:?} ...", &path);

    let file = OpenOptions::new()
        .create(false)
        .write(true)
        .read(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("opening {}, {}", path.as_os_str().to_str().unwrap(), e));

    let mut reader = BufReader::new(file);
    let mut buf = vec![0u8; 1024];
    let mut output_buffer = BufWriter::new(stdout());

    //Builder::new() .name(target_socket_addr.to_string()) .spawn(move || {
    while let Ok(c) = reader.read(&mut buf) {
        if c == 0 {
            //eprintln!("encountered zero-length message!");
            break;
        }

        //#[cfg(debug_assertions)]
        //println!("\n{} client: {:?}", c, String::from_utf8_lossy(&buf[..c]));
        for (target_addr, target_socket) in &targets {
            target_socket
                .send_to(&buf[0..c], &target_addr)
                .expect("sending to server socket");
        }
        //}) .unwrap()
        if tee {
            let o = output_buffer
                .write(&buf[0..c])
                .expect("writing to output buffer");
            output_buffer.flush().unwrap();
            assert!(c == o);
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            exit(1);
        }
    };

    //let mut threads = vec![];

    //for hostname in args.server_addr {
    //println!( "logging {}: listening for {}", &args.path.as_os_str().to_str().unwrap(), hostname);
    //threads.push(
    let _ = client_socket_stream(&args.path, args.server_addrs, args.tee);
    //);
    //}
}
