use crate::bonzomatic;
use crate::utils;
use futures_util::{future, pin_mut};
use futures_util::{SinkExt, StreamExt};
use glob::glob;
use log::info;
use rand::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::read_to_string;
use tokio::time::{sleep, Duration};

use tokio_tungstenite::connect_async;
fn credential_header(name: &str) -> String {
    String::from(format!(
        "\n/****\nRadio bonzo\n\nCredit: \n {name} \n****/\n"
    ))
}
fn playlist_from_path(path: String) -> Vec<PathBuf> {
    let mut playlist = Vec::new();

    for entry in glob(&path).unwrap() {
        match entry {
            Ok(path) => playlist.push(path),
            Err(e) => eprintln!("{e:?}"),
        }
    }
    playlist
}
async fn credited_source(filepath: &str) -> String {
    let contents = read_to_string(filepath).await.unwrap();
    let filepath: PathBuf = PathBuf::from(filepath);
    let filename = filepath.file_name().unwrap().to_str().unwrap();
    let end_of_first_line = contents.find("\n").unwrap() + 1;
    let (head_file, tail_file) = contents.split_at(end_of_first_line);
    let mut result_file = head_file.to_owned();
    result_file.push_str(&credential_header(filename));
    result_file.push_str(&tail_file);
    result_file.to_owned()
}
/// Radio mode
pub async fn radio(
    protocol: &str,
    host: &str,
    room: &str,
    handle: &str,
    path: &str,
    update_interval: &u64,
    time_per_entry: &u64,
) {
    // Prepare Websocket url
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    info!("Replay to {ws_url}");

    let (ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect");
    info!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();

    // We need to consume the read stream or the  websocket connection get interrupted
    let ws_read = { read.for_each(|_| async { () }) };

    let ws_write = async {
        let mut rng = rand::thread_rng();
        loop {
            let mut vec: Vec<PathBuf> = playlist_from_path(String::from(path));
            vec.shuffle(&mut rng);
            let start_time = SystemTime::now();
            info!("Radio");
            while vec.len() != 0 {
                let current = vec.pop().unwrap();
                info!("{:?}", current);
                let source = credited_source(current.to_str().unwrap()).await;
                // Prepare payload, only time will change later
                let mut payload = bonzomatic::Payload::from(
                    0u32,
                    0,
                    source,
                    true,
                    0,
                    String::from(handle),
                    String::from(room),
                    0f64,
                );
                for _ in 0..(time_per_entry / update_interval) {
                    let since_start = SystemTime::now()
                        .duration_since(start_time)
                        .expect("Time went backwards");
                    payload.update_shader_time(since_start.as_secs_f64());
                    let payload = payload.to_message();
                    write.send(payload).await.unwrap();
                    sleep(Duration::from_millis(*update_interval)).await;
                }
            }
        }
    };
    pin_mut!(ws_write, ws_read);
    future::select(ws_write, ws_read).await;
}
