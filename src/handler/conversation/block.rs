use chrono::Utc;
use serde_json::json;
use mongodb::bson::doc;
use uuid::Uuid;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::Conversation;

#[derive(Debug, Deserialize)]
pub struct ReqBody {
    user_id: String,
}

pub async fn task(
    req: HttpRequest,
    req_body: web::Json<ReqBody>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;
    let db = MongoDB.connect();

    // Check if already blocked
    let block_collection = db.collection::<Conversation::ConversationBlock>("conversation_block");
    let result = block_collection.find_one(doc!{
        "blocker_id": &user_id,
        "blocked_id": &req_body.user_id,
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();

    if let Some(existing) = option {
        // Already blocked — remove it
        let result = block_collection.delete_one(doc!{
            "blocker_id": &user_id,
            "blocked_id": &req_body.user_id,
        }).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        return Ok(
            HttpResponse::Ok()
            .content_type("application/json")
            .json(json!({
                "blocked_id": &req_body.user_id,
                "is_blocked": false
            }))
        )
    }

    // Not blocked — add it
    let favorite = Conversation::ConversationBlock {
        uuid: Uuid::new_v4().to_string(),
        blocker_id: user_id,
        blocked_id: req_body.user_id.clone(),
        blocked_at: Utc::now().timestamp_millis(),
    };

    let result = block_collection.insert_one(&favorite).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "blocked_id": &req_body.user_id,
            "is_blocked": true
        }))
    )
}
