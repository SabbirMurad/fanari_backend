use super::ImageStruct;
use serde::{Deserialize, Serialize};

//page_core
#[derive(Debug, Deserialize, Serialize)]
pub struct PageCore {
    pub uuid: String,
    pub owner: String, // user_id
    pub name: String,
    pub tag_name: String,
    pub anyone_can_join: bool,
    pub tags: Vec<String>,
    pub profile_picture: Option<ImageStruct>,
    pub biography: Option<String>,
    pub visibility: PageVisibility,

    pub modified_at: i64,
    pub created_at: i64,
}

//page_social
#[derive(Debug, Deserialize, Serialize)]
pub struct PageSocial {
    pub uuid: String,

    pub like_count: i64,
    pub follower_count: i64,
    pub block_count: i64,

    pub modified_at: i64,
}

//page_membership
#[derive(Debug, Deserialize, Serialize)]
pub struct PageMembership {
    pub uuid: String,
    pub admins: Vec<String>,
    pub members: Vec<String>,
    pub member_count: i64,

    pub modified_at: i64,
}

//page_like
#[derive(Debug, Deserialize, Serialize)]
pub struct PageLike {
    pub page_id: String,
    pub liked_by: String,
    pub liked_at: i64,
}

//page_follow
#[derive(Debug, Deserialize, Serialize)]
pub struct PageFollow {
    pub page_id: String,
    pub followed_by: String,
    pub followed_at: i64,
}

//page_block
#[derive(Debug, Deserialize, Serialize)]
pub struct PageBlocked {
    pub page_id: String,
    pub blocked: String,
    pub blocked_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageVisibility {
    Public,
    Private,
    Unlisted,
}
impl std::fmt::Display for PageVisibility {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt,"{:?}", self)
    }
}
  