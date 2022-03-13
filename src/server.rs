use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::{net::TcpListener, thread::spawn};
use tungstenite::protocol::WebSocket;
use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
};

pub fn run() {
    println!("lol");
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    let streams = Arc::new(RwLock::new(HashMap::new()));
    for stream in server.incoming() {
        let q = Arc::clone(&streams);
        spawn(move || {
            println!("spawn");
            let callback = |req: &Request, mut response: Response| {
                println!("Received a new ws handshake");
                println!("The request's path is: {}", req.uri().path());
                println!("The request's headers are:");
                for (ref header, value) in req.headers() {
                    println!("* {header} {value:?}");
                }

                // Let's add an additional header to our response to the client.
                /*
                let headers = response.headers_mut();
                headers.append("MyCustomHeader", ":)".parse().unwrap());
                headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());*/
                Ok(response)
            };

            let websocket = Arc::new(RwLock::new(accept_hdr(stream.unwrap(), callback).unwrap()));
            use uuid::Uuid;
            let uuid = Uuid::new_v4();
            {
                q.write().unwrap().insert(uuid, Arc::clone(&websocket));
            }
            loop {
                let mut cws = { websocket.write().unwrap() };
                let msg = cws.read_message().unwrap();
                let l = q.read().unwrap();
                for (k, ws) in &*l {
                    if *k != uuid {
                        let mut wss = ws.write().unwrap();
                        println!("m");
                        wss.write_message(msg.clone()).unwrap();
                    }
                }
                println!("{l:?}");
                /*if msg.is_binary() || msg.is_text() {
                    //websocket.write_message(msg).unwrap();
                }*/
            }
        });
    }
}
