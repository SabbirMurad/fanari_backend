use super::{ImageStruct, AudioStruct, Mention};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentStatus { Active, Deleted, Suspended }
impl std::fmt::Display for CommentStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//comment_core
#[derive(Debug, Deserialize, Serialize)]
pub struct CommentCore {
    pub uuid: String,
    pub owner: String,
    pub post_id: String,

    pub text: Option<String>,
    pub images: Vec<ImageStruct>,
    pub audio: Option<AudioStruct>,

    pub status: CommentStatus,
    pub is_edited: bool,
    pub mentions: Vec<Mention>,

    pub created_at: i64,
    pub modified_at: i64,
    pub deleted_at: Option<i64>,
    pub suspended_at: Option<i64>,
    pub suspended_by: Option<String>,
}

//comment_stat
#[derive(Debug, Deserialize, Serialize)]
pub struct CommentStat {
    pub uuid: String,

    pub like_count: i64,
    pub reply_count: i64,

    pub modified_at: i64,
}

//comment_like
#[derive(Debug, Deserialize, Serialize)]
pub struct CommentLike {
    pub comment_id: String,
    pub liked_by: String,
    pub liked_at: i64,
}