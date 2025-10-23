use serde::{Deserialize, Serialize};

pub mod account;
pub use account as Account;

pub mod documentation;
pub use documentation as Documentation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedImageType { Gif, Png, Jpeg, Webp }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageStruct {
  pub uuid: String,
  pub height: usize,
  pub width: usize,
  pub r#type: AllowedImageType
}