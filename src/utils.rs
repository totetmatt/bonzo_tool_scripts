pub fn get_ws_url(protocol: &String, host: &String, room: &String, handle: &String) -> String {
    format!("{protocol}://{host}/{room}/{handle}")
}

pub fn get_file_basename(room: &String, handle: &String, ts: &u128) -> String {
    format!("{room}_{handle}_{ts}")
}
