use chrono::Utc;
use serde_json::json;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use serde::Deserialize;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::Conversation;

#[derive(Debug, Deserialize)]
pub struct ReqBody {
    conversation_id: String,
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

    // Verify the user is a participant in this conversation
    let participant_collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");
    let result = participant_collection.find_one(doc!{
        "conversation_id": &req_body.conversation_id,
        "user_id": &user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    if result.unwrap().is_none() {
        return Ok(Response::not_found("conversation not found"));
    }

    // Check if already muted
    let mute_collection = db.collection::<Conversation::ConversationMuted>("conversation_muted");
    let result = mute_collection.find_one(doc!{
        "conversation_id": &req_body.conversation_id,
        "user_id": &user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let existing = result.unwrap();

    if existing.is_some() {
        // Already muted — unmute it
        let result = mute_collection.delete_one(doc!{
            "conversation_id": &req_body.conversation_id,
            "user_id": &user_id
        }).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        return Ok(
            HttpResponse::Ok()
            .content_type("application/json")
            .json(json!({
                "conversation_id": &req_body.conversation_id,
                "is_muted": false
            }))
        );
    }

    // Not muted — mute it
    let muted = Conversation::ConversationMuted {
        conversation_id: req_body.conversation_id.clone(),
        user_id: user_id.clone(),
        created_at: Utc::now().timestamp_millis(),
    };

    let result = mute_collection.insert_one(&muted).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "conversation_id": &req_body.conversation_id,
            "is_muted": true
        }))
    )
}
