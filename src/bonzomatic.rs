use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tungstenite::protocol::Message;
#[derive(Serialize, Deserialize, Debug)]
struct Data {
    #[serde(rename(serialize = "Anchor", deserialize = "Anchor"))]
    anchor: u32,
    #[serde(rename(serialize = "Caret", deserialize = "Caret"))]
    caret: u32,
    #[serde(rename(serialize = "Code", deserialize = "Code"))]
    code: String,
    #[serde(rename(serialize = "Compile", deserialize = "Compile"))]
    compile: bool,
    #[serde(rename(serialize = "FirstVisibleLine", deserialize = "FirstVisibleLine"))]
    first_visible_line: u32,
    #[serde(rename(serialize = "NickName", deserialize = "NickName"))]
    nickname: String,
    #[serde(rename(serialize = "RoomName", deserialize = "RoomName"))]
    room_name: String,
    #[serde(rename(serialize = "ShaderTime", deserialize = "ShaderTime"))]
    shader_time: f64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    #[serde(rename(serialize = "Data", deserialize = "Data"))]
    data: Data,
}
impl Payload {
    pub fn get_shader_time(&self) -> f64 {
        self.data.shader_time
    }
    pub fn get_caret(&self) -> u32 {
        self.data.caret
    }
    pub fn get_anchor(&self) -> u32 {
        self.data.anchor
    }
    pub fn get_compile(&self) -> bool {
        self.data.compile
    }
    pub fn get_code(&self) -> &String {
        &self.data.code
    }
    pub fn get_visible_line(&self) -> u32 {
        self.data.first_visible_line
    }
    pub fn get_nickname(&self) -> &String {
        &self.data.nickname
    }
    pub fn update_shader_time(&mut self, shader_time: f64) {
        self.data.shader_time = shader_time;
    }
    pub fn from_message(message: &Message) -> Self {
        let data = message.to_text().unwrap();
        let str_payload: String = data[0..data.len() - 1].to_string();
        serde_json::from_str(&str_payload).expect("Serde error")
    }
    pub fn from_str(message: &String) -> Self {
        serde_json::from_str(message).expect("Can't parse")
    }
    pub fn from(
        anchor: u32,
        caret: u32,
        code: String,
        compile: bool,
        first_visible_line: u32,
        nickname: String,
        room_name: String,
        shader_time: f64,
    ) -> Self {
        Self {
            data: Data {
                anchor: anchor,
                caret: caret,
                code: code,
                compile: compile,
                first_visible_line: first_visible_line,
                nickname,
                room_name: room_name,
                shader_time: shader_time,
            },
        }
    }
    pub fn to_str(&self) -> String {
        serde_json::to_string(self).expect("")
    }
    pub fn to_message(&self) -> Message {
        // \0 is needed a the end of the message by bonzomatic network
        Message::Text(self.to_str() + "\0")
    }
    async fn save_history(&self, dir_path: &PathBuf, filename: &String) {
        let msg = self.to_str();
        let mut dir_path = dir_path.clone();
        dir_path.push(format!("{}.json", filename.to_owned()));
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(dir_path)
            .await
            .unwrap();

        file.write_all(format!("{}\n", msg.to_owned()).as_bytes())
            .await
            .unwrap();
        file.sync_all().await.unwrap()
    }
    async fn save_current(&self, dir_path: &PathBuf, filename: &String) {
        let msg = &self.data.code;
        let mut dir_path = dir_path.clone();
        dir_path.push(format!("{}.glsl", filename.to_owned()));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // This is needed to "reset" the file size in case the current update is smaller than the previous one.
            .open(dir_path)
            .await
            .unwrap();

        file.write_all(msg.as_bytes()).await.unwrap();
        file.sync_all().await.unwrap()
    }
    pub async fn save(
        &self,
        save_glsl: bool,
        save_history: bool,
        only_compile: bool,
        dir_path: &PathBuf,
        filename: &String,
    ) {
        let save_current_future = self.save_current(&dir_path, &filename);
        let save_history_future = self.save_history(&dir_path, &filename);
        if save_glsl {
            tokio::join!(save_current_future,);
        }
        if save_history {
            if !only_compile || self.data.compile {
                tokio::join!(save_history_future,);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_saved_str() -> String {
        String::from(
            r###"{"Data":{"Anchor":0,"Caret":0,"Code":"#version 410 core\n\nuniform float fGlobalTime; // in seconds\nuniform vec2 v2Resolution; // viewport resolution (in pixels)\nuniform float fFrameTime; // duration of the last frame, in seconds\n\nuniform sampler1D texFFT; // towards 0.0 is bass / lower freq, towards 1.0 is higher / treble freq\nuniform sampler1D texFFTSmoothed; // this one has longer falloff and less harsh transients\nuniform sampler1D texFFTIntegrated; // this is continually increasing\nuniform sampler2D texPreviousFrame; // screenshot of the previous frame\nuniform sampler2D texChecker;\nuniform sampler2D texNoise;\nuniform sampler2D texTex1;\nuniform sampler2D texTex2;\nuniform sampler2D texTex3;\nuniform sampler2D texTex4;\nuniform sampler2D texTex5;\nuniform float fMidiKnob;\n\nlayout(location = 0) out vec4 out_color; // out_color must be written in order to see anything\n\nvec4 plas( vec2 v, float time )\n{\n\tfloat c = 0.5 + sin( v.x * 10.0 ) + cos( sin( time + v.y ) * 20.0 );\n\treturn vec4( sin(c * 0.2 + cos(time)), c * 0.15, cos( c * 0.1 + time / .4 ) * .25, 1.0 );\n}\nvoid main(void)\n{\n\tvec2 uv = vec2(gl_FragCoord.x / v2Resolution.x, gl_FragCoord.y / v2Resolution.y);\n\tuv -= 0.5;\n\tuv /= vec2(v2Resolution.y / v2Resolution.x, 1);\n\n\tvec2 m;\n\tm.x = atan(uv.x / uv.y) / 3.14;\n\tm.y = 1 / length(uv) * .2;\n\tfloat d = m.y;\n\n\tfloat f = texture( texFFT, d ).r * 100;\n\tm.x += sin( fGlobalTime ) * 0.1;\n\tm.y += fGlobalTime * 0.25;\n\n\tvec4 t = plas( m * 3.14, fGlobalTime ) / d;\n\tt = clamp( t, 0.0, 0.0 );\n\tout_color = f + t;\n}","Compile":false,"FirstVisibleLine":0,"NickName":"replay","RoomName":"livecode","ShaderTime":0.6100148558616638}}            "###,
        )
    }
    fn get_ws_message_str() -> String {
        String::from(
            r###"{
            "Data": {
                "Anchor": 10,
                "Caret": 12,
                "Code": "#version 410 core\n\nuniform float fGlobalTime; \/\/ in seconds\nuniform vec2 v2Resolution; \/\/ viewport resolution (in pixels)\nuniform float fFrameTime; \/\/ duration of the last frame, in seconds\n\nuniform sampler1D texFFT; \/\/ towards 0.0 is bass \/ lower freq, towards 1.0 is higher \/ treble freq\nuniform sampler1D texFFTSmoothed; \/\/ this one has longer falloff and less harsh transients\nuniform sampler1D texFFTIntegrated; \/\/ this is continually increasing\nuniform sampler2D texPreviousFrame; \/\/ screenshot of the previous frame\nuniform sampler2D texChecker;\nuniform sampler2D texNoise;\nuniform sampler2D texTex1;\nuniform sampler2D texTex2;\nuniform sampler2D texTex3;\nuniform sampler2D texTex4;\nuniform sampler2D texTex5;\nuniform float fMidiKnob;\n\nlayout(location = 0) out vec4 out_color; \/\/ out_color must be written in order to see anything\n\nvec4 plas( vec2 v, float time )\n{\n\tfloat c = 0.5 + sin( v.x * 10.0 ) + cos( sin( time + v.y ) * 20.0 );\n\treturn vec4( sin(c * 0.2 + cos(time)), c * 0.15, cos( c * 0.1 + time \/ .4 ) * .25, 1.0 );\n}\nvoid main(void)\n{\n\tvec2 uv = vec2(gl_FragCoord.x \/ v2Resolution.x, gl_FragCoord.y \/ v2Resolution.y);\n\tuv -= 0.5;\n\tuv \/= vec2(v2Resolution.y \/ v2Resolution.x, 1);\n\n\tvec2 m;\n\tm.x = atan(uv.x \/ uv.y) \/ 3.14;\n\tm.y = 1 \/ length(uv) * .2;\n\tfloat d = m.y;\n\n\tfloat f = texture( texFFT, d ).r * 100;\n\tm.x += sin( fGlobalTime ) * 0.1;\n\tm.y += fGlobalTime * 0.25;\n\n\tvec4 t = plas( m * 3.14, fGlobalTime ) \/ d;\n\tt = clamp( t, 0.0, 0.0 );\n\tout_color = f + t;\n}",
                "Compile": true,
                "FirstVisibleLine": 5,
                "NickName": "replay",
                "Parameters": {
                    "fMidiKnob": 0 
                },
                "RoomName": "livecode",
                "ShaderTime": 0.3068600594997406006 
            } 
        } 
         "###,
        )
    }
    fn get_ws_message() -> Message {
        Message::text(get_ws_message_str())
    }
    #[test]
    fn test_parse_from_message() {
        let test = get_ws_message();
        let result = Payload::from_message(&test);
        assert_eq!(result.data.anchor, 10);
        assert_eq!(result.data.compile, true);
        assert!(!result.data.code.is_empty());
        assert_eq!(result.data.room_name, "livecode");
        assert_eq!(result.data.nickname, "replay");
        assert_eq!(result.data.first_visible_line, 5);
    }
    #[test]
    fn test_parse_from_str() {
        let test = get_saved_str();
        let result = Payload::from_str(&test);
        assert_eq!(result.data.anchor, 0);
        assert_eq!(result.data.compile, false);
        assert!(!result.data.code.is_empty());
        assert_eq!(result.data.room_name, "livecode");
        assert_eq!(result.data.nickname, "replay");
        assert_eq!(result.data.first_visible_line, 0);
    }
    #[test]
    fn test_message_to_str_to_message() {
        let test = get_ws_message();
        let result = Payload::from_message(&test);
        let result_str = result.to_str();
        let result_from_str = Payload::from_str(&result_str);
        assert_eq!(result.data.code, result_from_str.data.code);
        assert_eq!(result.data.anchor, result_from_str.data.anchor);
        assert_eq!(result.data.compile, result_from_str.data.compile);
        assert_eq!(result.data.shader_time, result_from_str.data.shader_time);
    }
    #[test]
    #[should_panic]
    fn test_wrong_parse_message() {
        Payload::from_message(&Message::Text(String::from("test")));
    }

    #[test]
    #[should_panic]
    fn test_wrong_parse_str() {
        Payload::from_str(&String::from("test"));
    }
}
