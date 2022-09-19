use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::sync::{Arc, Barrier};
use std::thread::{Builder, JoinHandle};

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_socket, new_socket};

const HELP: &str = r#"
DISPATCH: SERVER

USAGE:
  server --path [OUTPUT_LOGFILE] --listen_addr [SOCKET_ADDR] ...

  e.g.
  server --path logfile.log --listen_addr 127.0.0.1:9920 --listen_addr [::1]:9921


FLAGS:
  -h, --help    Prints help information

"#;

struct ServerArgs {
    listen_addr: Vec<String>,
    path: String,
}

fn parse_args() -> Result<ServerArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
        print!("{}", HELP);
        exit(0);
    }
    let args = ServerArgs {
        path: pargs.value_from_str("--path")?,
        listen_addr: pargs.values_from_str("--listen_addr")?,
    };
    if args.listen_addr.is_empty() {
        eprintln!("Error: the --listen_addr option must be set. Must provide atleast one client IP address");
    };

    Ok(args)
}

/// server: client socket handler
/// binds a new socket connection on the network multicast channel
fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    // https://bluejekyll.github.io/blog/posts/multicasting-in-rust/
    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", addr.ip());
    match addr.ip() {
        IpAddr::V4(ref mdns_v4) => {
            let socket = new_socket(&addr)?;
            // join multicast channel on all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            let bind_result = bind_socket(&socket, &addr);
            if bind_result.is_err() {
                panic!("binding to {:?}  {:?}", addr, bind_result);
            }

            Ok(socket.into())
        }
        IpAddr::V6(ref mdns_v6) => {
            #[cfg(debug_assertions)]
            println!("mdns_v6: {}", mdns_v6);
            let socket = match new_socket(&addr) {
                Ok(s) => s,
                Err(e) => panic!("creating new socket {}", e),
            };
            // bind to all interfaces
            //assert!(socket.set_multicast_if_v6(0).is_ok());

            // join multicast channel
            assert!(socket.join_multicast_v6(mdns_v6, 0).is_ok());
            //socket.join_multicast_v6(&Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), addr.port().into())?;

            // disable ipv4->ipv6 multicast rerouting
            assert!(socket.set_only_v6(true).is_ok());

            let listenaddr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                //IpAddr::V6(Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110)),
                addr.port(),
            );
            let bind_result = bind_socket(&socket, &listenaddr);
            if bind_result.is_err() {
                panic!("binding to {:?}  {:?}", listenaddr, bind_result);
            }

            Ok(socket.into())
        }
    }
}

pub fn join_unicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(&addr)?;
    bind_socket(&socket, &addr)?;
    Ok(socket.into())
}

/// server socket listener
pub fn listener(addr: String, logfile: PathBuf) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let upstream_barrier = Arc::new(Barrier::new(2));
    let downstream_barrier = Arc::clone(&upstream_barrier);

    let addr: SocketAddr = addr
        .parse()
        .unwrap_or_else(|e| panic!("parsing socket address '{}' {}", addr, e));

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&logfile);
    let mut writer = match file {
        Ok(f) => f,
        Err(e) => {
            panic!("{:?}: {}", &logfile, e);
        }
    };
    //let mut writer = BufWriter::with_capacity(1024, file);
    //let listen_socket = match addr.ip().is_multicast() || multicast {
    let listen_socket = match addr.ip().is_multicast() {
        false => join_unicast(addr).expect("failed to create socket listener!"),
        true => {match join_multicast(addr) {
            Ok(s) => s,
            Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
        }},
    };
    let join_handle = Builder::new()
        .name(format!("{}:server", addr))
        .spawn(move || {
            #[cfg(debug_assertions)]
            println!("{}:server: joined", addr);

            //#[cfg(test)]
            upstream_barrier.wait();

            #[cfg(debug_assertions)]
            println!("{}:server: is ready", addr);

            let mut buf = [0u8; 1024]; // receive buffer
            loop {
                match listen_socket.recv_from(&mut buf[0..]) {
                    Ok((c, _remote_addr)) => {
                        /*
                        #[cfg(debug_assertions)]
                        println!(
                        "{}:server: got {} bytes from {}\t\t{}",
                        addr,
                        c,
                        _remote_addr,
                        String::from_utf8_lossy(&buf[0..c]),
                        );
                        */

                        let _ = writer
                            .write(&buf[0..c])
                            .unwrap_or_else(|_| panic!("writing to {:?}", &logfile));
                        //buf = [0u8; 1024];

                        /*
                        let responder = new_socket(&remote_addr).expect("failed to create responder");
                        let remote_socket = SockAddr::from(remote_addr);
                        responder .send_to(thread_name.as_bytes(), &remote_socket) .expect("failed to respond");
                        #[cfg(debug_assertions)]
                        println!( "{}:server: sent thread_name {} to: {}", thread_name, thread_name, _remote_addr);
                        */
                    }
                    Err(err) => {
                        writer.flush().unwrap();
                        eprintln!("{}:server: got an error: {}", addr, err);
                        #[cfg(debug_assertions)]
                        panic!("{}:server: got an error: {}", addr, err);
                    }
                }
            }
        })
        .unwrap();

    downstream_barrier.wait();
    #[cfg(debug_assertions)]
    println!("server: complete");
    join_handle
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

    let mut threads = vec![];

    let append_listen_addr = args.listen_addr.len() > 1;

    for hostname in args.listen_addr {
        // if listening to multiple clients at once, log each client to a
        // separate file, with the client IP appended to the filename
        let mut logpath: String = "".to_owned();
        logpath.push_str(&args.path);
        if append_listen_addr {
            for pathsegment in [&args.path, &".".to_string(), &hostname] {
                logpath.push_str(pathsegment);
            }
        }

        println!("logging transmissions from {} to {}", hostname, logpath);
        threads.push(listener(hostname, PathBuf::from_str(&logpath).unwrap()));
    }
    for thread in threads {
        let _ = thread.join().unwrap();
    }
}
