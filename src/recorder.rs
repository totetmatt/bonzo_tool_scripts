extern crate ws;
use std::io::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};
// "drone.alkama.com:9000/livecode/radio"

use ws::connect;
use ws::util::Token;
use ws::{Error, Handler, Handshake, Message, Result};
/// Recorder for bonzomatic network websocket instance

fn format_name(room: &String, handle: &String, ts: &u128) -> String {
    format!("{}_{}_{}", room, handle, ts)
}

pub struct Client {
    handle: String,
    out_file: std::fs::File,
    cpt: i32,
}
impl Client {
    pub fn init(protocol: &String, host: &String, room: &String, handle: &String) {
        // Get useful data and formated data
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let basename_id = format_name(&room, &handle, &ts);

        // Prepare file
        let filename = &format!("{basename_id}.json");
        let path = std::path::Path::new(filename);
        let file = &std::fs::File::create(&path).expect("Error creating file");

        // Connect to websocket
        let url = format!("{protocol}://{host}/{room}/{handle}");
        println!("Connection to {}", url);
        connect(url, move |_| Client {
            handle: basename_id.clone(),
            out_file: file.try_clone().expect("Error Coling File"),
            cpt: 0,
        })
        .unwrap()
    }
}
impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("Connection Open");
        Ok(())
    }
    fn on_message(&mut self, msg: Message) -> Result<()> {
        self.cpt += 1;
        let handle = &self.handle;
        let cpt = &self.cpt;
        println!("{handle}:{cpt}");
        let txt = indoc! { msg.as_text().expect("Erorr")};
        writeln!(self.out_file, "{txt}").expect("Error writing Json to zip");
        Ok(())
    }
    fn on_error(&mut self, err: Error) {
        eprintln!("Error {err}")
    }
    fn on_timeout(&mut self, _: Token) -> Result<()> {
        Ok(eprintln!("Timeout"))
    }
}
