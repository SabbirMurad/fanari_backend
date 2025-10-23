use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationType {
    Like,
    Comment,
    Mention,
    Follow,
    FriendRequest,
    FriendAccept,
    Replied,
    Tag,
    Shared,
    SystemAlert,
}
impl std::fmt::Display for NotificationType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub uuid: String,            // Unique ID for the notification
    pub recipient_id: String,    // Who the notification is for
    pub sender_id: Option<String>, // Who triggered it (if applicable)
    
    pub n_type: NotificationType,
    pub message: Option<String>,   // Optional readable message
    pub metadata: Option<serde_json::Value>,
    // Optional extra data (post_id, comment_id, etc.)

    pub read: bool,                // Whether the user has seen it
    pub seen_at: Option<i64>,      // When user opened the notifications page
    pub created_at: i64,
}