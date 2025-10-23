use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Documentation {
    pub uuid: String,
    pub name: String,
    pub view_count: i64,
    pub created_at: i64,
    pub created_by: String,
    pub deleted_at: Option<i64>,
    pub deleted_by: Option<String>,
}