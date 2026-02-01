use serde::{Deserialize, Serialize};
use super::{ImageStruct, VideoStruct, AudioStruct, AttachmentStruct, Mention};

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
    pub created_at: i64,
}

//group_conversation
#[derive(Debug, Deserialize, Serialize)]
pub struct GroupConversation {
    pub uuid: String,
    pub owner: String,
    pub admins: Vec<String>,
    pub members: Vec<String>,
    pub profile_picture: String,
    pub name: String,
}

//single_conversation
#[derive(Debug, Deserialize, Serialize)]
pub struct SingleConversation {
    pub uuid: String,
    pub user_1: String,
    pub user_2: String,
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
    pub seen_by: Vec<String>,
    pub created_at: i64
}

//message_text
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageText {
    pub uuid: String,
    pub text: String,
    pub mentions: Vec<Mention>
}

//message_emoji
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageEmoji {
    pub uuid: String,
    pub emoji: String,
}

//message_image
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageImage {
    pub uuid: String,
    pub images: Vec<ImageStruct>,
}

//message_audio
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageAudio {
    pub uuid: String,
    pub audio: AudioStruct,
}

//message_video
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageVideo {
    pub uuid: String,
    pub video: VideoStruct,
}

//message_attachment
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageAttachment {
    pub uuid: String,
    pub video: VideoStruct,
    pub attachment: AttachmentStruct,
}