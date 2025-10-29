use serde::{Deserialize, Serialize};
use super::{ VideoStruct, AudioStruct, Mention};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostOwnerType { User, Page }
impl std::fmt::Display for PostOwnerType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostStatus { Active, Deleted, Suspended }
impl std::fmt::Display for PostStatus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostVisibility { Public, FieldsOnly, FriendAndFollowers }
impl std::fmt::Display for PostVisibility {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}

//post_core
#[derive(Debug, Deserialize, Serialize)]
pub struct PostCore {
    pub uuid: String,
    pub owner: String,

    pub caption: Option<String>,
    pub images: Vec<String>,
    pub videos: Vec<VideoStruct>,
    pub audio: Option<AudioStruct>,
    pub mentions: Vec<Mention>,

    pub owner_type: PostOwnerType,
    pub visibility: PostVisibility,
    pub tags: Vec<String>,
    pub is_nsfw: bool,
    pub content_warning: Option<String>,

    pub created_at: i64,
    pub modified_at: i64,
    pub deleted_at: Option<i64>,
    pub suspended_at: Option<i64>,
    pub suspended_by: Option<String>,
}

//post_stat
#[derive(Debug, Deserialize, Serialize)]
pub struct PostStat {
    pub uuid: String,

    pub like_count: i64,
    pub comment_count: i64,
    pub share_count: i64,
    pub view_count: i64,

    pub modified_at: i64,
}

//post_like
#[derive(Debug, Deserialize, Serialize)]
pub struct PostLike {
    pub post_id: String,
    pub liked_by: String,
    pub liked_at: i64,
}