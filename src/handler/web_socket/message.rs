use actix::prelude::{Message, Recipient};
use serde_json::Value;
use serde::{Deserialize, Serialize};

use crate::model::conversation::ConversationType;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum WsEnvelopeType {
    text,
    typing,
    connect,
    disconnect,
    new_conversation,
    call_signal,
    message_seen,
}

impl std::fmt::Display for WsEnvelopeType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WsEnvelope {
    #[serde(rename = "type")]
    pub msg_type: WsEnvelopeType,
    pub payload: Value,  // stays as raw JSON until we know the type
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessage>,
    pub rooms: Vec<String>,
    pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub rooms: Vec<String>,
    pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
    pub user_id: String,
    pub msg: WsEnvelope,
    pub room_id: String,
}

// For 1-to-1 call signaling (Offer/Answer/IceCandidate need a specific target)
#[derive(Message)]
#[rtype(result = "()")]
pub struct DirectMessage {
    pub from_user_id: String,
    pub to_user_id: String,
    pub msg: WsEnvelope,
}

// ✅ NEW — For group call control messages (join/leave/toggle)
// Broadcasts to everyone in the room EXCEPT the sender
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomSignalMessage {
    pub from_user_id: String,
    pub room_id: String,
    pub msg: WsEnvelope,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddToRoom {
    pub user_id: String,
    pub conversation_id: String,
    pub conversation_type: ConversationType,
}