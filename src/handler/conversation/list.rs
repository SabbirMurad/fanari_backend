use chrono::Utc;
use uuid::Uuid;
use serde_json::json;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{
    Conversation,
    Account,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ascending: Option<bool>
}

pub async fn task(req: HttpRequest, req_query: web::Query<ReqQuery>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let db = MongoDB.connect();
    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");

    let result = utils::mongo::find_with_pagination(
        &collection,
        doc!{},
        req_query.ascending,
        req_query.limit,
        req_query.offset,
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();

    
    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "core": serde_json::to_value(&conversation).unwrap(),
            "single_payload": serde_json::to_value(&conversation_details).unwrap(),
        }))
    )
}