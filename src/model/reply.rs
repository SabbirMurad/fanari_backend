use serde::{Deserialize, Serialize};
use super::{AudioStruct, Mention};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplyStatus { Active, Deleted, Suspended }
impl std::fmt::Display for ReplyStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//reply_core
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyCore {
    pub uuid: String,
    pub owner: String,
    pub comment_id: String,

    pub text: Option<String>,
    pub images: Vec<String>,
    pub audio: Option<AudioStruct>,

    pub status: ReplyStatus,
    pub mentions: Vec<Mention>,
    pub is_edited: bool,

    pub created_at: i64,
    pub modified_at: i64,
    pub deleted_at: Option<i64>,
    pub suspended_at: Option<i64>,
    pub suspended_by: Option<String>,
}

//reply_stat
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyStat {
    pub uuid: String,

    pub like_count: i64,
    pub modified_at: i64,
}

//reply_like
#[derive(Debug, Deserialize, Serialize)]
pub struct ReplyLike {
    pub reply_id: String,
    pub liked_by: String,
    pub liked_at: i64,
}