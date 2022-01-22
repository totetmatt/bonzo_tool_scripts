use crate::utils;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek};
use std::{thread, time};
use tungstenite::{connect, Message};
pub fn replay(protocol: &str, host: &str, room: &str, handle: &str, filename: &str) {
    // Prepare Websocket url
    let ws_url = utils::get_ws_url(protocol, host, room, handle);
    println!("Replay to {ws_url}");

    // Connect to websocket entrypoint
    let (mut socket, response) = connect(&ws_url).expect("Can't connect");
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

        socket.write_message(Message::Text(line.into())).unwrap();

        println!("{filename} {current_idx}/{nb_lines} > {ws_url}");
        thread::sleep(time::Duration::from_millis(300)); // To parameterize
    }
}
