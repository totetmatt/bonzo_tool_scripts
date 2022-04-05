mod wsbonzoendpoint;

use crate::bonzomatic;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use wsbonzoendpoint::WsBonzoEndpoint;

use log::{debug, info, warn};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::fs::create_dir_all;

use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<SocketAddr, Tx>>>;
type InstanceMap = Arc<RwLock<HashMap<SocketAddr, Arc<WsBonzoEndpoint>>>>;

struct FileSaveMessage {
    message: Message,
    meta: Arc<WsBonzoEndpoint>,
    ts: u128,
}

async fn handle_connection(
    peer_map: PeerMap,
    instance_map: InstanceMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    sender: Option<Sender<FileSaveMessage>>,
) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    info!("Incoming TCP connection from: {}", addr);
    let mut endpoint = Arc::new(WsBonzoEndpoint::empty());
    let callback = |req: &Request, response: Response| {
        match WsBonzoEndpoint::parse_resource(req.uri().path()) {
            Ok(e) => endpoint = Arc::new(e),
            Err(e) => info!("{e}"),
        }
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);
    info!("{endpoint:?}");
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
        match &sender {
            Some(s) => {
                let send_msg_to_save_queue = s.try_send(FileSaveMessage {
                    message: msg.clone(),
                    meta: Arc::clone(&endpoint),
                    ts: ts,
                });
                tokio::spawn(async { send_msg_to_save_queue });
            }
            None => (),
        };

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
    info!("{} disconnected", &addr);
    {
        peer_map.write().unwrap().remove(&addr);
        instance_map.write().unwrap().remove(&addr);
    }
}

async fn save_message(mut crx: Receiver<FileSaveMessage>, dir_path: PathBuf) {
    while let Some(message) = crx.recv().await {
        match message.message {
            Message::Ping(_) => debug!("Ping"),
            Message::Text(_) => {
                let payload: bonzomatic::Payload =
                    bonzomatic::Payload::from_message(&message.message);
                match message.meta.filename(&message.ts) {
                    Ok(filename) => {
                        payload.save(&dir_path, &filename).await;
                    }
                    Err(_) => {
                        warn!("Error, not valid entrypoint for saving to file");
                    }
                }
            }

            _ => (),
        }
    }
}
use std::path::PathBuf;
pub async fn main(addr: &String, save_shader_disable: &bool, save_shader_dir: &PathBuf) -> () {
    let state = PeerMap::new(RwLock::new(HashMap::new()));
    let instances = InstanceMap::new(RwLock::new(HashMap::new()));
    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(addr.to_owned()).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);
    let sender = if !save_shader_disable {
        let (ctx, crx) = mpsc::channel::<FileSaveMessage>(256);
        info!("Save shaders in {}", save_shader_dir.display());
        create_dir_all(save_shader_dir).await.unwrap();
        tokio::spawn(save_message(crx, save_shader_dir.to_owned()));
        Some(ctx)
    } else {
        info!("Not saving shaders");
        None
    };
    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            state.clone(),
            instances.clone(),
            stream,
            addr,
            sender.as_ref().map(|x| x.clone()),
        ));
    }
}
