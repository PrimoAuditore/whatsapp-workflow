use std::collections::HashMap;
use fizzy_commons::shared_structs::MessageRequest;
use serde::{Deserialize, Serialize};
use crate::constants::{FlowStatusId, MessageType, ResponseStatus};

#[derive(Serialize, Deserialize)]
pub struct Event {
    object: String,
    pub(crate) entry: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
pub struct MediaData {
    pub url: String,
    pub mime_type: String,
    pub sha256: String,
    pub file_size: i32,
    pub id: String,
    pub messaging_product: String,
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    id: String,
    pub(crate) changes: Vec<Change>,
}

#[derive(Serialize, Deserialize)]
pub struct Change {
    field: String,
    pub(crate) value: ChangeValue,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeValue {
    messaging_product: String,
    metadata: ChangeMetadata,
    contacts: Option<Vec<Contact>>,
    pub(crate) messages: Option<Vec<Message>>,
    pub statuses: Option<Vec<Status>>,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    id: String,
    status: String,
    timestamp: String,
    recipient_id: String,
    conversation: Option<Conversation>,
}

#[derive(Serialize, Deserialize)]
pub struct Conversation {
    id: String,
    origin: Origin,
}

#[derive(Serialize, Deserialize)]
pub struct Origin {
    #[serde(alias = "type")]
    origin_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeMetadata {
    display_phone_number: String,
    phone_number_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Contact {
    profile: Profile,
    wa_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub(crate) context: Option<Context>,
    pub(crate) from: String,
    pub(crate) id: String,
    pub(crate) timestamp: String,

    #[serde(alias = "type")]
    pub(crate) message_type: String,
    pub image: Option<Image>,
    pub(crate) text: Option<Text>,
    pub(crate) button: Option<Button>,
    pub(crate) interactive: Option<Interactive>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Image {
    pub caption: String,
    pub mime_type: String,
    pub sha256: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Interactive {
    #[serde(alias = "type")]
    interactive_type: String,
    pub(crate) list_reply: Option<ListReply>,
    pub button_reply: Option<ListReply>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListReply {
    pub(crate) id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Button {
    payload: String,
    pub(crate) text: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Context {
    from: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Text {
    pub(crate) body: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListChoice {
    pub title: String,
    pub id: String,
}

impl ListChoice {
    pub fn new() -> ListChoice {
        ListChoice {
            id: "".to_string(),
            title: "".to_string(),
        }
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = title.to_string();
        self
    }

    pub fn id(&mut self, id: &str) -> &mut Self {
        self.id = id.to_string();
        self
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageLog {
    pub timestamp: String,
    pub destination_systems: Vec<String>,
    pub origin_system: String,
    pub phone_number: String,
    pub origin: String,
    pub register_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StandardResponse {
    pub references: Vec<ModifiedReference>,
    pub errors: Option<Vec<String>>,
}

impl StandardResponse {
    pub fn new() -> StandardResponse {
        StandardResponse {
            references: vec![],
            errors: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModifiedReference {
    pub(crate) system: String,
    pub(crate) reference: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RequestTracker{
    pub(crate) phone_number: String,
    pub(crate) timestamp: String,
    pub(crate) id: String

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackerStep{
    pub(crate) tracker_id: String,
    pub(crate) timestamp: String,
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) value: String,
    pub(crate) attached_files: String,
    pub(crate) message_reference: String,

}

// #[derive(Serialize, Deserialize, Clone)]
// pub struct MessageRequest {
//     pub system_id: u8,
//     pub to: Vec<String>,
//     pub message_type: String,
//     pub content: MessageContent,
// }
// #[derive(Serialize, Deserialize, Clone)]
// pub struct MessageContent {
//     pub body: Option<String>,
//     pub list: Option<ListMessage>,
//     pub buttons: Option<ButtonMessage>,
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct ListMessage {
//     pub(crate) title: String,
//     pub(crate) choices: Vec<String>,
// }

#[derive(Serialize, Deserialize, Clone)]
pub struct ButtonMessage {
    pub title: String,
    pub choices: Vec<String>,
}


pub struct StepDefinition {
    // Requirements to create this step
    pub(crate) required_response: Option<MessageType>, // Required response type in order to create a step
    pub(crate) validation_regex: Option<String>, // Required body regex in order to create a step

    // Behaviour in case the step can be created
    pub(crate) next_step: Option<FlowStatusId>, // Next step depending on this step definition
    pub(crate) successful_response: Option<MessageRequest>, // Response to user in case the step can be created
    pub(crate) data_origin: Option<String>, // Origin of redis data for lists and button replies
}
