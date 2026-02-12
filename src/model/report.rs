use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
  Threat,
  Spam,
  Inappropriate,
  HateTowardReligion,
  Other
}

impl std::fmt::Display for ReportType {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportedOn {
  Profile,
  Post,
  Comment,
  Reply
}

impl std::fmt::Display for ReportedOn {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStatus { Pending, Investigating, Resolved }
impl std::fmt::Display for ReportStatus {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Report {
  pub uuid: String,
  pub owner: String,
  pub r#type: ReportType,
  pub status: ReportStatus,
  pub reported_on: ReportedOn,
  pub reported_uuid: String,
  pub reason: String,
  pub reply: Option<String>,
  pub resolved_at: Option<i64>,
  pub resolved_by: Option<String>,
  pub created_at: i64,
}