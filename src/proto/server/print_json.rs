use serde::Serialize;
use serde_repr::Serialize_repr;

#[derive(Serialize, Clone, Debug)]
pub struct PrintJson {
    pub data: Vec<JsonMessagePart>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<AdditionalInfo>,
}

impl PrintJson {
    pub fn chat_message(text: impl Into<String>) -> Self {
        Self {
            data: vec![JsonMessagePart::chat_message(text)],
            additional_info: Some(AdditionalInfo {
                r#type: Some(Type::Chat),
                ..<_>::default()
            }),
        }
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct AdditionalInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<NetworkItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub countdown: Option<u32>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct JsonMessagePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_status: Option<HintStatus>,
}

impl JsonMessagePart {
    pub fn chat_message(text: impl Into<String>) -> Self {
        JsonMessagePart {
            text: Some(text.into()),
            ..Self::default()
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub enum Type {
    ItemSend,
    ItemCheat,
    Hint,
    Join,
    Part,
    Chat,
    ServerChat,
    Tutorial,
    TagsChanged,
    CommandResult,
    AdminCommandResult,
    Goal,
    Release,
    Collect,
    Countdown,
}

#[derive(Serialize, Clone, Debug)]
pub struct NetworkItem {
    pub item: u32,
    pub location: u32,
    pub player: u32,
    pub flags: u32,
}

#[derive(Serialize_repr, Clone, Debug)]
#[repr(u32)]
pub enum HintStatus {
    Unspecified = 0,
    NoPriority = 10,
    Avoid = 20,
    Priority = 30,
    Found = 40,
}
