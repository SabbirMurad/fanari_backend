use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportStatus {
  Pending,
  Ongoing,
  Closed,
}

impl std::fmt::Display for SupportStatus {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}

// support
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Report {
  pub uuid: String,
  pub ticket: i64,
  pub owner: String,
  pub status: SupportStatus,
  pub created_at: i64,
}

//