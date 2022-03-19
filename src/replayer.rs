use crate::bonzomatic;
use crate::utils;
use futures_util::{future, pin_mut};
use futures_util::{SinkExt, StreamExt};
use tokio::time::{sleep, Duration};
use tokio::{
    fs::File,
    io::{
        // This trait needs to be imported, as the lines function being
        // used on reader is defined there
        AsyncBufReadExt,
        BufReader,
    },
};

use log::info;
use std::time::SystemTime;
use tokio_tungstenite::connect_async;
pub async fn replay(
    protocol: &str,
    host: &str,
    room: &str,
    handle: &str,
    filename: &str,
    update_interval: &u64,
) {
    let start_time = SystemTime::now();
    // Prepare Websocket url
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    info!("Replay to {ws_url}");
    let tokio_file = File::open(filename).await.unwrap();
    // create reader using file
    let reader = BufReader::new(tokio_file);
    // get iterator over lines
    let mut lines = reader.lines();

    let (ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();

    // We need to consume the read stream or the  websocket connection get interrupted
    let ws_read = { read.for_each(|_| async { () }) };

    let ws_write = async {
        while let Some(line) = lines.next_line().await.expect("Failed to read file") {
            let since_start = SystemTime::now()
                .duration_since(start_time)
                .expect("Time went backwards");
            let mut payload: bonzomatic::Payload = bonzomatic::Payload::from_str(&line);
            payload.update_shader_time(since_start.as_secs_f64());
            let payload = payload.to_message();
            write.send(payload).await.unwrap();
            sleep(Duration::from_millis(*update_interval)).await;
        }
    };
    pin_mut!(ws_write, ws_read);
    future::select(ws_write, ws_read).await;
}
