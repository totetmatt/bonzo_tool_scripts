use crate::utils;
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
use std::sync::Mutex;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;
pub async fn bridge(protocol: &str, host: &str, from_room: &str, to_room: &str, handle: &str) {
    // Prepare Websocket url
    let from_ws_url = utils::get_ws_url(protocol, host, from_room, handle);
    info!("Record from  {from_ws_url}");

    let (from_ws_stream, _) = connect_async(from_ws_url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");
    let (_, read) = from_ws_stream.split();

    let to_ws_url = utils::get_ws_url(protocol, host, to_room, handle);
    info!("Record to {to_ws_url}");

    let (to_ws_stream, _) = connect_async(to_ws_url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");
    let (write, _) = to_ws_stream.split();

    let write_ptr = Mutex::new(write);
    let ws_read = {
        read.for_each(|message| async {
            match message {
                Ok(data) => match data {
                    Message::Ping(_) => debug!("Ping!"),
                    Message::Text(_) => {
                        let _ = write_ptr.lock().unwrap().send(data).await.unwrap();
                    }
                    _ => (),
                },
                _ => (),
            }
        })
    };
    ws_read.await
}
