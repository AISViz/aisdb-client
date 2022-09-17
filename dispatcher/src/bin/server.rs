use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;

#[path = "../socket.rs"]
pub mod socket;
use socket::{bind_socket, new_socket};

/// server: client socket handler
/// binds a new socket connection on the network multicast channel
fn join_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    // https://bluejekyll.github.io/blog/posts/multicasting-in-rust/
    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", addr.ip());
    //assert!(addr.ip().is_multicast() || addr.ip().);
    match addr.ip() {
        IpAddr::V4(ref mdns_v4) => {
            let socket = new_socket(&addr)?;
            // join multicast channel on all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            //socket.bind(&SockAddr::from(addr))?;
            bind_socket(&socket, &addr)?;
            //println!("server listening on addr: {}", addr);

            Ok(socket.into())
        }
        IpAddr::V6(ref mdns_v6) => {
            #[cfg(debug_assertions)]
            println!("mdns_v6: {}", mdns_v6);
            let socket = new_socket(&addr)?;
            // bind to all interfaces
            //assert!(socket.set_multicast_if_v6(0).is_ok());

            // join multicast channel
            assert!(socket.join_multicast_v6(mdns_v6, 0).is_ok());
            //socket.join_multicast_v6(&Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), addr.port().into())?;

            // disable ipv4->ipv6 multicast rerouting
            assert!(socket.set_only_v6(true).is_ok());

            //let bind_result = socket.bind(&SockAddr::from(addr));
            //let socketv6addr = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), addr.port(), 0, 0);
            //let bindaddr = SockAddr::from(socketv6addr);
            //println!("server listening on addr: {:?}", bindaddr);
            //let bind_result = socket.bind(&bindaddr);
            let listenaddr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                //IpAddr::V6(Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0110)),
                //addr.ip().into(),
                //IpAddr::V6(*mdns_v6),
                addr.port(),
            );
            let bind_result = bind_socket(&socket, &listenaddr);
            if bind_result.is_err() {
                panic!("binding to {:?}  {:?}", addr, bind_result);
            }

            Ok(socket.into())
        }
    }
}

pub fn join_unicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let socket = new_socket(&addr)?;
    socket.bind(&socket2::SockAddr::from(addr))?;
    Ok(socket.into())
}

/// server socket listener
pub fn listener(
    response: String,
    addr: SocketAddr,
    logfile: PathBuf,
    multicast: bool,
) -> JoinHandle<()> {
    // A barrier to not start the client test code until after the server is running
    let upstream_barrier = Arc::new(Barrier::new(2));
    let downstream_barrier = Arc::clone(&upstream_barrier);

    //let file = File::open(&logfile).unwrap_or_else(|_| panic!("opening {:?}", &logfile));
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
    let listener = match addr.ip().is_multicast() || multicast {
        false => join_unicast(addr).expect("failed to create socket listener!"),
        true => {match join_multicast(addr) {
            Ok(s) => s,
            Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
        }},
    };

    let join_handle = std::thread::Builder::new()
        .name(format!("{}:server", response))
        .spawn(move || {
            #[cfg(debug_assertions)]
            println!("{}:server: joined: {}", response, addr);

            #[cfg(test)]
            upstream_barrier.wait();

            #[cfg(debug_assertions)]
            println!("{}:server: is ready", response);

            let mut buf = [0u8; 1024]; // receive buffer
            loop {
                match listener.recv_from(&mut buf[0..]) {
                    Ok((c, _remote_addr)) => {
                        /*
                        #[cfg(debug_assertions)]
                        println!(
                        "\n{}:server: got bytes: {} from: {}\nmsg: {}",
                        response,
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
                        // create a socket to send the response
                        let responder =
                        new_socket(&remote_addr).expect("failed to create responder");

                        let remote_socket = SockAddr::from(remote_addr);

                        // we send the response that was set at the method beginning
                        responder
                        .send_to(response.as_bytes(), &remote_socket)
                        .expect("failed to respond");

                        #[cfg(debug_assertions)]
                        println!(
                        "{}:server: sent response {} to: {}",
                        response, response, _remote_addr
                        );
                        */
                    }
                    Err(err) => {
                        writer.flush().unwrap();
                        eprintln!("{}:server: got an error: {}", response, err);
                        #[cfg(debug_assertions)]
                        panic!("{}:server: got an error: {}", response, err);
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

/// ensure the server is stopped
pub struct NotifyServer(pub Arc<AtomicBool>);
impl Drop for NotifyServer {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct ServerArgs {
    client_addr: Vec<String>,
    path: String,
    port: u16,
}

fn parse_args() -> Result<ServerArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    /*
    if pargs.contains(["-h", "--help"]) || pargs.clone().finish().is_empty() {
    print!("{}", HELP);
    std::process::exit(0);
    }
    */
    let args = ServerArgs {
        port: pargs.value_from_str("--port")?,
        path: pargs.value_from_str("--path")?,
        client_addr: pargs.values_from_str("--client_addr")?,
        //pargs .opt_value_from_str("--listen_addr")? .unwrap_or("127.0.0.1".to_string()),
    };

    Ok(args)
}

pub fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    for hostname in args.client_addr {
        let hostaddr = IpAddr::from_str(&hostname).unwrap();
        let socketaddr = SocketAddr::new(hostaddr, args.port);
        let mut logpath: String = "".to_owned();

        for pathsegment in [
            &args.path,
            &socketaddr.ip().to_string(),
            &".".to_string(),
            &socketaddr.port().to_string(),
        ] {
            logpath.push_str(pathsegment);
        }

        println!(
            "logging transmissions from {} to {}",
            hostaddr.to_string(),
            logpath
        );
        listener(
            hostaddr.to_string(),
            socketaddr,
            PathBuf::from_str(&logpath).unwrap(),
            socketaddr.ip().is_multicast(),
        );
    }
}
