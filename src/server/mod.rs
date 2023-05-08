mod wsbonzoendpoint;

use crate::bonzomatic;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::SinkExt;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::{debug, info, warn};
use serde_json::json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::fs::create_dir_all;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::Duration;
use tungstenite::handshake::server::{Request, Response};
use tungstenite::protocol::Message;
use wsbonzoendpoint::WsBonzoEndpoint;
type Tx = UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<SocketAddr, Tx>>>;
type InstanceMap = Arc<RwLock<HashMap<SocketAddr, Arc<WsBonzoEndpoint>>>>;
type EndpointLockMap = Arc<RwLock<HashMap<Arc<WsBonzoEndpoint>, SocketAddr>>>;
struct FileSaveMessage {
    message: Message,
    meta: Arc<WsBonzoEndpoint>,
    ts: u128,
}

async fn handle_connection(
    peer_map: PeerMap,
    instance_map: InstanceMap,
    endpoint_lock: EndpointLockMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    sender: Option<Sender<FileSaveMessage>>,
) -> Result<(), tungstenite::Error> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    info!("Incoming TCP connection from: {addr}");
    let mut endpoint = Arc::new(WsBonzoEndpoint::empty());
    let callback = |req: &Request, response: Response| {
        info!("{:?}", req.uri());
        match WsBonzoEndpoint::parse_resource(req.uri().path()) {
            Ok(e) => endpoint = Arc::new(e),
            Err(e) => info!("{e}"),
        }
        Ok(response)
    };

    let ws_stream = tokio_tungstenite::accept_hdr_async(raw_stream, callback)
        .await?;
    info!("WebSocket connection established: {addr}");
    info!("{endpoint:?}");
    info!("{}", endpoint.room);
    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    let (mut outgoing, incoming) = ws_stream.split();

    if !endpoint.is_info {
        {
            instance_map
                .write()
                .unwrap()
                .insert(addr, Arc::clone(&endpoint));

            peer_map.write().unwrap().insert(addr, tx);
        } // release locks

        let broadcast_incoming = incoming.try_for_each(|msg| {
            // There is a ping/pong message sent even if you're grabber
            // It needs to be discarded with a OK message to not disconnect the grabber
            if !msg.is_binary() && !msg.is_text() {
                return future::ok(());
            }

            {
                // Making sure only one user is streaming to one path
                // First come first save.
                let mut write_enpoint_lock = endpoint_lock.write().unwrap();
                match write_enpoint_lock.get(&endpoint) {
                    Some(a) if *a != addr => {
                        return future::err(tungstenite::Error::ConnectionClosed)
                    }
                    _ => write_enpoint_lock.insert(Arc::clone(&endpoint), addr),
                };
            }
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
    } else {
        // Room entry point where you get reports of current connected user
        println!("{endpoint:?}");
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        let ws_read = { incoming.for_each(|_| async { () }) };
        let ws_write = async {
            loop {
                interval.tick().await;
                let m: Vec<WsBonzoEndpoint> = endpoint_lock
                    .read()
                    .unwrap()
                    .keys()
                    .map(|a| (**a).clone())
                    .filter(|a| endpoint.room == "" || a.room == endpoint.room) // Filter if it matches room or if empty return everything
                    .collect();
                let m = Message::Text(json!(m).to_string() + "\0"); // Bonzomatic needs \0
                outgoing.send(m).await.unwrap();
            }
        };
        pin_mut!(ws_write, ws_read);
        future::select(ws_write, ws_read).await;
    }
    info!("{} disconnected", &addr);
    {
        peer_map.write().unwrap().remove(&addr);
        instance_map.write().unwrap().remove(&addr);
        // When disconnecting, release the path slot
        let mut write_enpoint_lock = endpoint_lock.write().unwrap();
        match write_enpoint_lock.get(&endpoint) {
            Some(a) if *a == addr => write_enpoint_lock.remove(&Arc::clone(&endpoint)),
            _ => None,
        };
    }
    Ok(())
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

pub async fn main(addr: &String, save_shader_disable: &bool, save_shader_dir: &PathBuf) -> () {
    let state = PeerMap::new(RwLock::new(HashMap::new()));
    let instances = InstanceMap::new(RwLock::new(HashMap::new()));
    let endpoint_lock = EndpointLockMap::new(RwLock::new(HashMap::new()));
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
    // tokio::spawn(handle_p(endpoint_lock.clone()));
    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            state.clone(),
            instances.clone(),
            endpoint_lock.clone(),
            stream,
            addr,
            sender.as_ref().map(|x| x.clone()),
        ));
    }
}
