use super::Mention;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationType { Single, Group, }
impl std::fmt::Display for ConversationType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//conversation_favorite
#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationFavorite {
    pub conversation_id: String,
    pub user_id: String,
    pub created_at: i64,
}

//conversation_core
#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationCore {
    pub uuid: String,
    pub r#type: ConversationType,
    pub last_message_at: i64,
    pub last_message_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationRole { Owner, Admin, Member }
impl std::fmt::Display for ConversationRole {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//conversation_participant
#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationParticipant {
    pub conversation_id: String,
    pub user_id: String,
    pub role: ConversationRole,
    pub joined_at: i64,
    pub last_message_read_id: Option<String>,
    pub is_favorite: bool,
    pub is_muted: bool,
}

//group_conversation_metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct GroupConversationMetadata {
    pub conversation_id: String,
    pub name: String,
    pub image: Option<String>,
    pub owner_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType { Text, Emoji, Image, Audio, Video, Attachment }
impl std::fmt::Display for TextType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//message_core
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageCore {
    pub uuid: String,
    pub conversation_id: String,
    pub owner: String,
    pub r#type: TextType,
    pub reply_to: Option<String>,
    pub created_at: i64,
}

//message_content
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageContent {
    pub message_id: String,

    pub text: Option<String>,
    pub mentions: Option<Vec<Mention>>,
    pub emoji: Option<String>,
    pub images: Option<Vec<String>>,
    pub audio: Option<String>,
    pub video: Option<String>,
    pub attachment: Option<String>,
}

//message_read
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageRead {
    pub message_id: String,
    pub user_id: String,
    pub read_at: i64,
}