use serde::{Deserialize, Serialize};
use super::{ VideoStruct, AudioStruct};

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
    pub videos: Vec<String>,
    pub audio: Option<AudioStruct>,
    pub poll: Option<String>,

    pub owner_type: PostOwnerType,
    pub visibility: PostVisibility,
    pub is_nsfw: bool,
    pub content_warning: Option<String>,

    pub created_at: i64,
    pub modified_at: i64,
    pub deleted_at: Option<i64>,
    pub suspended_at: Option<i64>,
    pub suspended_by: Option<String>,
}

//post_mention
#[derive(Debug, Deserialize, Serialize)]
pub struct PostMention {
    pub post_id: String,
    pub user_id: String,
    pub start: usize,
    pub end: usize,
}

//post_tag
#[derive(Debug, Deserialize, Serialize)]
pub struct PostTag {
    pub post_id: String,
    pub tag: String,
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

//post_bookmark
#[derive(Debug, Deserialize, Serialize)]
pub struct PostBookmark {
    pub post_id: String,
    pub bookmarked_by: String,
    pub bookmarked_at: i64,
}