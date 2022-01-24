use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
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
    shader_time:  f64
}
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Payload {
    #[serde(rename(serialize = "Data", deserialize = "Data"))]
    data: Data
 
}
impl Payload {
    pub fn update_shader_time(&mut self , shader_time:f64 ) {
        self.data.shader_time = shader_time;
    }
}
