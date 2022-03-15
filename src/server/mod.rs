pub mod bonzoendpoint;
use bonzoendpoint::BonzoEndpoint;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<SocketAddr, Tx>>>;
type InstanceMap = Arc<RwLock<HashMap<SocketAddr, Arc<BonzoEndpoint>>>>;

struct FileSaveMessage {
    message: Message,
    meta: Arc<BonzoEndpoint>,
}

async fn handle_connection(
    peer_map: PeerMap,
    instance_map: InstanceMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    sender: Sender<FileSaveMessage>,
) {
    println!("Incoming TCP connection from: {}", addr);
    let mut endpoint = Arc::new(BonzoEndpoint::empty());
    let callback = |req: &Request, response: Response| {
        match BonzoEndpoint::parse_resource(req.uri().path()) {
            Ok(e) => endpoint = Arc::new(e),
            Err(e) => println!("{e}"),
        }
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);
    println!("{endpoint:?}");
    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    {
        instance_map
            .write()
            .unwrap()
            .insert(addr, Arc::clone(&endpoint));
        peer_map.write().unwrap().insert(addr, tx);
    } // release locks
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let send_msg_to_save_queue = sender.try_send(FileSaveMessage {
            message: msg.clone(),
            meta: Arc::clone(&endpoint),
        });
        tokio::spawn(async { send_msg_to_save_queue });

        let peers = peer_map.read().unwrap();
        let instance = instance_map.read().unwrap();

        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| {
                peer_addr != &&addr && endpoint.can_send_to(instance.get(&peer_addr).unwrap())
            })
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });
    let receive_from_others = rx.map(Ok).forward(outgoing);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    println!("{} disconnected", &addr);
    {
        peer_map.write().unwrap().remove(&addr);
        instance_map.write().unwrap().remove(&addr);
    }
}

async fn save_history(filename: &String, msg: Message) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .await
        .unwrap();

    file.write_all((msg.into_text().unwrap() + &"\n").as_bytes())
        .await
        .unwrap();
    file.sync_all().await.unwrap()
}
async fn save_current(filename: &String, msg: Message) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("last_".to_owned() + filename)
        .await
        .unwrap();

    file.write_all(msg.to_text().unwrap().as_bytes())
        .await
        .unwrap();
    file.sync_all().await.unwrap()
}
async fn save_message_in_file(mut crx: Receiver<FileSaveMessage>) {
    while let Some(message) = crx.recv().await {
        match message.meta.json_filename() {
            Ok(filename) => {
                tokio::join!(
                    save_current(&filename, message.message.clone()),
                    save_history(&filename, message.message.clone()),
                );
            }
            Err(_) => {
                eprintln!("Error, not valid entrypoint for saving to file");
            }
        }
    }
}

pub async fn main(addr: &String) -> () {
    let state = PeerMap::new(RwLock::new(HashMap::new()));
    let instances = InstanceMap::new(RwLock::new(HashMap::new()));
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(addr.to_owned()).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);
    let (ctx, crx) = mpsc::channel::<FileSaveMessage>(32);
    tokio::spawn(save_message_in_file(crx));
    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            state.clone(),
            instances.clone(),
            stream,
            addr,
            ctx.clone(),
        ));
    }
}
