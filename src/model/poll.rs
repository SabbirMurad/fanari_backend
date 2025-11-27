use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PollType { Single, Multiple }

impl std::fmt::Display for PollType {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt,"{:?}", self)
  }
}


//poll
#[derive(Debug, Deserialize, Serialize)]
pub struct Poll {
    pub uuid: String,
    pub question: String,
    pub options: Vec<String>,
    pub r#type: PollType,
}

//poll_vote
#[derive(Debug, Deserialize, Serialize)]
pub struct PollVote {
    pub poll_id: String,
    pub user_id: String,
    pub option: String,
}