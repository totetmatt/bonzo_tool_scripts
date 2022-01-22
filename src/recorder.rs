use crate::utils;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tungstenite::connect;

// "drone.alkama.com:9000/livecode/radio"
/// Recorder for bonzomatic network websocket instance

pub fn record(protocol: &str, host: &str, room: &str, handle: &str) {
    // Get useful data and formated data
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let basename_id = utils::get_file_basename(room, handle, &ts);

    // Prepare file
    let filename = &format!("{basename_id}.json");
    let path = std::path::Path::new(filename);
    let mut file = &std::fs::File::create(&path).expect("Error creating file");

    //WS Connect, as listener / client only
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    let (mut socket, response) = connect(&ws_url).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }
    let mut update_received_count: u128 = 0;
    loop {
        let msg = socket.read_message().expect("Error reading message");
        // One json per line
        // Can't really serde, as bonzomatic sends a final `\0` that most of parser will consider as error
        let msg = msg.into_text().expect("ser").replace("\n", "");
        println!("{basename_id}:{update_received_count}");
        writeln!(file, "{msg}").expect("Error writing Json to zip");
        update_received_count += 1
    }
}
