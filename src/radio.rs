use glob::glob;
use std::path::PathBuf;

use rand::prelude::*;
use std::{thread, time};

use std::fs;

use crate::bonzomatic;
use crate::utils;
use bonzomatic::Payload;
use std::time::SystemTime;
use tungstenite::client::connect_with_config;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Message;
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
fn credential_header(name: &str) -> String {
    String::from(format!(
        "\n/****\nRadio bonzo\n\nCredit: \n {name} \n****/\n"
    ))
}
fn credited_source(filepath: &str) -> String {
    let filepath: PathBuf = PathBuf::from(filepath);
    let filename = filepath.file_name().unwrap().to_str().unwrap();
    let contents = fs::read_to_string(filepath.to_str().unwrap())
        .expect("Something went wrong reading the file");
    let end_of_first_line = contents.find("\n").unwrap() + 1;
    let (head_file, tail_file) = contents.split_at(end_of_first_line);
    let mut result_file = head_file.to_owned();
    result_file.push_str(&credential_header(filename));
    result_file.push_str(&tail_file);
    result_file.to_owned()
}
pub fn radio(
    protocol: &str,
    host: &str,
    room: &str,
    handle: &str,
    path: &str,
    update_interval: &u64,
    time_per_entry: &u64,
) {
    let mut rng = rand::thread_rng();
    loop {
        let mut vec: Vec<PathBuf> = playlist_from_path(String::from(path));
        vec.shuffle(&mut rng);
        let start_time = SystemTime::now();

        while vec.len() != 0 {
            let current = vec.pop();
            println!("{:?}", current);
            let source = credited_source(current.unwrap().to_str().unwrap());

            // Prepare Websocket url
            let ws_url = utils::get_ws_url(protocol, host, room, handle);
            println!("Sending to to {ws_url}");

            // Connect to websocket entrypoint
            let (mut socket, _) = connect_with_config(
                &ws_url,
                Some(WebSocketConfig {
                    max_send_queue: None,
                    max_message_size: None,
                    max_frame_size: None,
                    accept_unmasked_frames: true,
                }),
                10,
            )
            .expect("Can't connect");
            // Prepare payload, only time will change later
            let mut payload = Payload::from(
                0u32,
                0,
                source,
                true,
                0,
                String::from("radio"),
                String::from(room),
                0f64,
            );
            // Bonzomatic net asks to send regular
            for _ in 0..(time_per_entry / update_interval) {
                let since_start = SystemTime::now()
                    .duration_since(start_time)
                    .expect("Time went backwards");
                payload.update_shader_time(since_start.as_secs_f64());
                let payload = serde_json::to_string(&payload).expect("Can' t serialize");
                let payload = payload + "\0"; // needed by Bonzomatic

                socket.write_message(Message::Text(payload)).expect("err");
                #[warn(unused_must_use)]
                socket.read_message().unwrap();

                let sleep_time = time::Duration::from_millis(*update_interval);
                thread::sleep(sleep_time);
            }
        }
    }
}
