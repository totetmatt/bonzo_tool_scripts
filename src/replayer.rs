extern crate ws;
use std::fs::File;
use std::io::{self, BufRead};
use std::thread;
use ws::{connect, CloseCode, Error, Handler, Handshake, Result, Sender};
struct Server {
    out: Sender,
    file: String,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        let file = File::open(self.file.clone()).expect("Can't open file");
        for l in io::BufReader::new(file).lines() {
            let l = l.expect("Can't read line");
            self.out.send(l);
            thread::sleep_ms(300)
        }
        self.out.close(CloseCode::Normal)
    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

pub fn replay(protocol: &String, host: &String, room: &String, handle: &String, file: &String) {
    let entrypoint = format!("{protocol}://{host}/{room}/{handle}");

    println!("Replay to {entrypoint}");
    connect(entrypoint, |out| Server {
        out: out,
        file: file.clone(),
    })
    .unwrap()
}
