use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InvalidPacket {
    pub r#type: PacketProblemType,
    pub original_cmd: Option<String>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PacketProblemType {
    Known(KnownPacketProblemType),
    Unknown(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum KnownPacketProblemType {
    Cmd,
    Arguments,
}
