use serde::{Serialize, Serializer};
use serde_repr::Serialize_repr;

use crate::game::{ItemId, LocationId, SlotId};
use crate::proto::server::NetworkItem;

#[derive(Serialize, Clone, Debug)]
pub struct PrintJson {
    pub data: Vec<JsonMessagePart>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub additional_info: Option<AdditionalInfo>,
}

impl PrintJson {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    pub fn chat_message(text: impl Into<String>) -> Self {
        Self {
            data: vec![JsonMessagePart::chat_message(text)],
            additional_info: Some(AdditionalInfo {
                r#type: Some(Type::Chat),
                ..<_>::default()
            }),
        }
    }

    pub fn chat_message_for_received_item(item: NetworkItem, receiving_slot: SlotId) -> PrintJson {
        let mut message = PrintJson::builder().with_player(item.player);

        if item.player == receiving_slot {
            message.add_text(" found their ");
            message.add_net_item(item);
        } else {
            message.add_text(" sent ");
            message.add_net_item(item.with_player(receiving_slot));
        }

        message.add_text(" (");
        message.add_location(item.player, item.location);
        message.add_text(")");

        message.build_item_send(receiving_slot, item)
    }
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct AdditionalInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiving: Option<SlotId>,
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

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonMessagePart {
    Text {
        text: String,
    },
    ItemId {
        #[serde(rename = "text", serialize_with = "item_id_as_string")]
        item_id: ItemId,
        player: SlotId,
        flags: u64,
    },
    LocationId {
        #[serde(rename = "text", serialize_with = "location_id_as_string")]
        location_id: LocationId,
        player: SlotId,
    },
    PlayerId {
        #[serde(rename = "text", serialize_with = "slot_id_as_string")]
        player_id: SlotId,
    },
}

fn item_id_as_string<S: Serializer>(item_id: &ItemId, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&item_id.0.to_string())
}

fn slot_id_as_string<S: Serializer>(slot_id: &SlotId, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&slot_id.0.to_string())
}

fn location_id_as_string<S: Serializer>(
    location_id: &LocationId,
    ser: S,
) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&location_id.0.to_string())
}

// #[derive(Serialize, Clone, Debug, Default)]
// pub struct JsonMessagePart {
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub r#type: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub text: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub color: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub flags: Option<u32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub player: Option<u32>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub hint_status: Option<HintStatus>,
// }

impl JsonMessagePart {
    pub fn chat_message(text: impl Into<String>) -> Self {
        JsonMessagePart::Text { text: text.into() }
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

#[derive(Serialize_repr, Clone, Debug)]
#[repr(u32)]
pub enum HintStatus {
    Unspecified = 0,
    NoPriority = 10,
    Avoid = 20,
    Priority = 30,
    Found = 40,
}

pub struct MessageBuilder {
    parts: Vec<JsonMessagePart>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn add_text(&mut self, text: impl Into<String>) {
        self.parts.push(JsonMessagePart::Text { text: text.into() });
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.add_text(text);
        self
    }

    pub fn add_net_item(&mut self, item: NetworkItem) {
        self.add_item(item.player, item.item, item.flags);
    }

    pub fn add_item(&mut self, slot: SlotId, item: ItemId, flags: u64) {
        self.parts.push(JsonMessagePart::ItemId {
            item_id: item,
            player: slot,
            flags,
        });
    }

    pub fn with_item(mut self, slot: SlotId, item: ItemId, flags: u64) -> Self {
        self.add_item(slot, item, flags);
        self
    }

    pub fn add_location(&mut self, slot: SlotId, location: LocationId) {
        self.parts.push(JsonMessagePart::LocationId {
            location_id: location,
            player: slot,
        });
    }

    pub fn with_location(mut self, slot: SlotId, location: LocationId) -> Self {
        self.add_location(slot, location);
        self
    }

    pub fn add_player(&mut self, slot: SlotId) {
        self.parts
            .push(JsonMessagePart::PlayerId { player_id: slot });
    }

    pub fn with_player(mut self, slot: SlotId) -> Self {
        self.add_player(slot);
        self
    }

    pub fn build(self) -> PrintJson {
        PrintJson {
            data: self.parts,
            additional_info: None,
        }
    }

    pub fn build_item_send(self, slot_receiving: SlotId, item: NetworkItem) -> PrintJson {
        PrintJson {
            data: self.parts,
            additional_info: Some(AdditionalInfo {
                receiving: Some(slot_receiving),
                item: Some(item),
                ..Default::default()
            }),
        }
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
