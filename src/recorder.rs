//! Recorder Mode

use crate::bonzomatic;
use crate::utils;
use futures_util::StreamExt;
use log::{debug, info};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;

/// Record function
pub async fn record(protocol: &str, host: &str, room: &str, handle: &str) {
    // Prepare Websocket url
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let basename_id = utils::get_file_basename(room, handle, &ts);
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    info!("Record from to {ws_url}");

    let (ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    let (_, read) = ws_stream.split();
    let ws_read = {
        read.for_each(|message| async {
            match message {
                Ok(data) => match data {
                    Message::Ping(_) => debug!("Ping!"),
                    Message::Text(_) => {
                        let payload: bonzomatic::Payload = bonzomatic::Payload::from_message(&data);
                        payload.save(&PathBuf::from("./"), &basename_id).await;
                    }
                    _ => (),
                },
                _ => (),
            }
        })
    };
    ws_read.await
}
