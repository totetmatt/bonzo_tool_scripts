pub fn get_ws_url(protocol: &str, host: &str, room: &str, handle: &str) -> String {
    format!("{protocol}://{host}/{room}/{handle}")
}

pub fn get_file_basename(room: &str, handle: &str, ts: &u128) -> String {
    format!("{room}_{handle}_{ts}")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_basename() {
        let result = get_file_basename("livecode", "user", &1337u128);
        assert_eq!(result, "livecode_user_1337");
    }
    #[test]
    fn test_get_ws_url() {
        let result = get_ws_url("ws", "somehost", "aroom", "auser");
        assert_eq!(result, "ws://somehost/aroom/auser");

        let result = get_ws_url("ws", "localhost:9000", "aroom", "auser");
        assert_eq!(result, "ws://localhost:9000/aroom/auser");

        let result = get_ws_url("ws", "127.0.0.1:9000", "aroom", "auser");
        assert_eq!(result, "ws://127.0.0.1:9000/aroom/auser");
    }
}
