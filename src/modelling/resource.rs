use crate::SPVariable;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MessageCategory {
    OutGoing,
    Incoming,
    Service,
    Action
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub msgs: Vec<Message>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Message {
    pub name: String,
    pub topic: String,
    pub category: MessageCategory,
    pub message_type: String, // as json so that it can be serialized later
    pub variables: Vec<SPVariable>,
    pub variables_response: Vec<SPVariable>,
    pub variables_feedback: Vec<SPVariable>,
}

