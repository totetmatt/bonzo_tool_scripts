use crate::utils;
use crate::bonzomatic;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::time::SystemTime;
use std::{thread, time};
use tungstenite::{ Message};
use tungstenite::client::connect_with_config;
use tungstenite::protocol::WebSocketConfig;

pub fn replay(protocol: &str, host: &str, room: &str, handle: &str, filename: &str) {
    let start_time = SystemTime::now();
    // Prepare Websocket url
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    println!("Replay to {ws_url}");

    // Connect to websocket entrypoint
    let (mut socket, response) = connect_with_config(&ws_url,Some(WebSocketConfig{
        max_send_queue:None,
        max_message_size:None,max_frame_size:None,accept_unmasked_frames:true}),10).expect("Can't connect");
    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");

    // Open File
    let file = File::open(&filename).expect("Can't open file");
    let mut buffer: BufReader<File> = BufReader::new(file);
    let buffer = buffer.by_ref();
    let nb_lines = buffer.lines().count(); // Maybe more effective way ?
    buffer.rewind().expect("Error during buffer rewind");

    for (current_idx, line) in buffer.lines().enumerate() {
        let current_idx = current_idx + 1; // Non Zero count
        let line: &String = &line.expect("Can't read line");
        let mut payload :  bonzomatic::Payload= serde_json::from_str(line).expect("Can't parse");
        let since_start = SystemTime::now()
        .duration_since(start_time)
        .expect("Time went backwards");
        payload.update_shader_time(since_start.as_secs_f64());

        let payload = serde_json::to_string(&payload).expect("Can' t serialize");
        let payload = payload+"\0"; // needed by Bonzomatic

        socket.write_message(Message::Text(payload)).expect("err");
        #[warn(unused_must_use)]
        socket.read_message();
        
        println!("{filename} {current_idx}/{nb_lines} > {ws_url}");
        thread::sleep(time::Duration::from_millis(300)); // To parameterize
    }
}
