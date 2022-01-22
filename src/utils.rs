pub fn get_ws_url(protocol: &str, host: &str, room: &str, handle: &str) -> String {
    format!("{protocol}://{host}/{room}/{handle}")
}

pub fn get_file_basename(room: &str, handle: &str, ts: &u128) -> String {
    format!("{room}_{handle}_{ts}")
}
