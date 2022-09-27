use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread::spawn;
use std::time::{SystemTime, UNIX_EPOCH};

extern crate tungstenite;
use tungstenite::{accept, Message};

extern crate dispatcher;
use dispatcher::proxy::proxy_thread;
use dispatcher::reverse_proxy::ReverseProxyArgs;
use dispatcher::server::join_multicast;

extern crate nmea_parser;
use nmea_parser::{NmeaParser, ParsedMessage};

extern crate serde;
use serde::{Deserialize, Serialize};

extern crate serde_json;
use serde_json::to_string;

#[derive(Serialize, Deserialize)]
struct VesselPositionPing {
    mmsi: u32,
    lon: f64,
    lat: f64,
    time: u64,
    rot: f64,
    sog: f64,
    heading: f64,
}

pub fn filter_vesseldata(sentence: &str, parser: &mut NmeaParser) -> Option<String> {
    #[cfg(debug_assertions)]
    println!("{:?}", sentence);

    match parser.parse_sentence(sentence).ok()? {
        ParsedMessage::VesselDynamicData(vdd) => {
            let ping = VesselPositionPing {
                mmsi: vdd.mmsi,
                lon: vdd.longitude.unwrap(),
                lat: vdd.latitude.unwrap(),
                time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                rot: vdd.rot.unwrap_or(-1.),
                sog: vdd.sog_knots.unwrap_or(-1.),
                heading: vdd.heading_true.unwrap_or(-1.),
            };

            let msg = to_string(&ping).unwrap();
            Some(msg)
        }
        _ => None,
    }
}

fn process_message(buf: &[u8], i: usize, parser: &mut NmeaParser) -> Vec<Message> {
    let msg_txt = &String::from_utf8(buf[0..i].to_vec()).unwrap();
    let mut msgs = vec![];
    for msg in msg_txt.split("\r\n") {
        if msg.len() == 0 {
            continue;
        }
        if let Some(txt) = filter_vesseldata(msg, parser) {
            msgs.push(Message::Text(txt));
        }
    }
    msgs
}

fn handle_client(downstream: &TcpStream, multicast_addr: String) {
    let multicast_addr = multicast_addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("parsing socket address");
    if !multicast_addr.ip().is_multicast() {
        panic!("not a multicast address {}", multicast_addr);
    }
    let multicast_socket = join_multicast(multicast_addr).unwrap_or_else(|e| {
        panic!("joining multicast socket {}", e);
    });

    let mut buf = [0u8; 32768];
    let mut websocket = accept(downstream).unwrap();
    /*
    if msg.is_binary() || msg.is_text() { websocket.write_message(msg).unwrap(); } }
    */
    let mut parser = NmeaParser::new();

    loop {
        match multicast_socket.recv_from(&mut buf[0..]) {
            Ok((count_input, _remote_addr)) => {
                for msg in process_message(&buf, count_input, &mut parser) {
                    if let Err(e) = websocket.write_message(msg) {
                        eprintln!("dropping client: {}", e.to_string());
                        return;
                    }
                }
            }
            Err(err) => {
                eprintln!("stream_server upstream: got an error: {}", err);
                return;
            }
        }
    }
}

fn main() {
    let args = ReverseProxyArgs {
        udp_listen_addr: "[::]:9921".into(),
        tcp_listen_addr: "[::]:9920".into(),
        multicast_addr: "224.0.0.20:9919".into(),
        tee: false,
    };

    let _multicast = proxy_thread(
        &args.udp_listen_addr,
        &[args.multicast_addr.clone()],
        args.tee,
    );
    let listener = TcpListener::bind(args.tcp_listen_addr).unwrap();
    for stream in listener.incoming() {
        let multicast_addr = args.multicast_addr.clone();
        spawn(move || {
            handle_client(&stream.unwrap(), multicast_addr);
        });
    }
}
