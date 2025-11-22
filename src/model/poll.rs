use serde::{Deserialize, Serialize};

//poll
#[derive(Debug, Deserialize, Serialize)]
pub struct Poll {
    pub uuid: String,
    pub question: String,
    pub options: Vec<String>,
}

//poll_vote
#[derive(Debug, Deserialize, Serialize)]
pub struct PollVote {
    pub poll_id: String,
    pub user_id: String,
    pub option: String,
}