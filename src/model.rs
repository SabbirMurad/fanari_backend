use serde::{Deserialize, Serialize};

pub mod account;
pub use account as Account;


pub mod chat;
pub use chat as Chat;

pub mod post;
pub use post as Post;

pub mod page;
pub use page as Page;

pub mod comment;
pub use comment as Comment;

pub mod reply;
pub use reply as Reply;

pub mod notification;
pub use notification as Notification;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedImageType { Gif, Png, Jpeg, Webp }

impl std::fmt::Display for AllowedImageType {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      AllowedImageType::Gif => write!(fmt, "image/gif"),
      AllowedImageType::Png => write!(fmt, "image/png"),
      AllowedImageType::Jpeg => write!(fmt, "image/jpeg"),
      AllowedImageType::Webp => write!(fmt, "image/webp"),
    }
  }
}

impl AllowedImageType {
  pub fn to_str(&self) -> &str {
    match self {
      AllowedImageType::Gif => "image/gif",
      AllowedImageType::Png => "image/png",
      AllowedImageType::Jpeg => "image/jpeg",
      AllowedImageType::Webp => "image/webp",
    }
  }

  pub fn from_str(s: &str) -> AllowedImageType {
    match s {
      "image/gif" => AllowedImageType::Gif,
      "image/png" => AllowedImageType::Png,
      "image/jpeg" => AllowedImageType::Jpeg,
      "image/webp" => AllowedImageType::Webp,
      _ => AllowedImageType::Jpeg
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetUsedAt {
  ProfilePic,
  CoverPic,
  Post,
  Comment,
  Chat, 
  VideoThumbnail
}

impl std::fmt::Display for AssetUsedAt {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}

impl AssetUsedAt {
  pub fn from_str(s: &str) -> AssetUsedAt {
    match s {
      "ProfilePic" => AssetUsedAt::ProfilePic,
      "CoverPic" => AssetUsedAt::CoverPic,
      "Post" => AssetUsedAt::Post,
      "Comment" => AssetUsedAt::Comment,
      "Chat" => AssetUsedAt::Chat,
      "VideoThumbnail" => AssetUsedAt::VideoThumbnail,
      _ => AssetUsedAt::ProfilePic
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageStruct {
  pub uuid: String,
  pub height: usize,
  pub width: usize,
  pub original_size: usize,
  pub webp_size: usize,
  pub blur_hash: String,
  pub used_at: AssetUsedAt,
  pub original_type:  String,
  pub temporary: bool,
  pub deleted: bool,
  pub created_at: i64
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VideoStruct {
  pub uuid: String,
  pub height: usize,
  pub width: usize,
  pub thumbnail_type: AllowedImageType,
  pub length: i64,
  pub size: usize,
  pub used_at: AssetUsedAt,
  pub created_at: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AudioStruct {
  pub uuid: String,
  pub length: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AttachmentStruct {
  pub uuid: String,
  pub size: i64,
  pub name: String,
  pub extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mention {
    pub user_id: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedEmojiType { Gif, Png }

impl std::fmt::Display for AllowedEmojiType {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      AllowedEmojiType::Gif => write!(fmt, "image/gif"),
      AllowedEmojiType::Png => write!(fmt, "image/png"),
    }
  }
}