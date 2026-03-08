use futures::StreamExt;
use serde_json::{json, Value};
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{Conversation, ImageStruct};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqQuery {
    conversation_id: String,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn task(req: HttpRequest, req_query: web::Query<ReqQuery>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;
    let conversation_id = &req_query.conversation_id;

    let db = MongoDB.connect();

    // Verify user is a participant of this conversation
    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");

    let result = collection.find_one(doc!{
        "conversation_id": conversation_id,
        "user_id": &user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Ok(Response::not_found("Conversation not found"));
    }

    // Fetch message cores for the conversation
    let collection = db.collection::<Conversation::MessageCore>("message_core");

    let limit = req_query.limit.unwrap_or(20) as i64;
    let offset = req_query.offset.unwrap_or(0) as i64;

    let result = collection.find(doc!{
        "conversation_id": conversation_id
    }).sort(doc! {
        "created_at": -1
    })
    .limit(limit)
    .skip(offset as u64).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();

    let mut response = Vec::new();

    while let Some(message_core) = cursor.next().await {
        if let Err(error) = message_core {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let message_core = message_core.unwrap();

        let text = get_text(&message_core).await;
        match text {
            Ok(text) => response.push(text),
            Err(error) => return Ok(error),
        }
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(response)
    )
}

async fn get_text(text_core: &Conversation::MessageCore) -> Result<Value, HttpResponse> {
    let db = MongoDB.connect();

    let collection = db.collection::<Conversation::MessageContent>("message_content");

    let result = collection.find_one(doc!{
        "message_id": &text_core.uuid
    }).await;

    if let Err(error) = result {
        return Err(Response::internal_server_error(
            &error.to_string()
        ));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found(
            "Message content not found"
        ));
    }

    let text_content = option.unwrap();

    let collection = db.collection::<Conversation::MessageRead>("message_read");

    let result = collection.find(doc!{
        "message_id": &text_core.uuid
    }).await;

    if let Err(error) = result {
        return Err(Response::internal_server_error(
            &error.to_string()
        ));
    }

    let mut seen_by = Vec::new();
    let mut cursor = result.unwrap();
    while let Some(result) = cursor.next().await {
        if let Err(error) = result {
            return Err(Response::internal_server_error(
                &error.to_string()
            ));
        }

        let read = result.unwrap();

        seen_by.push(read.user_id);
    }

    let mut images: Option<Vec<ImageStruct>> = None;

    if !text_content.images.is_none() {
        let mut images_some = Vec::new();

        for image in text_content.images.unwrap().iter() {
            let collection = db.collection::<ImageStruct>("image");
    
            let result = collection.find_one(doc!{
                "uuid": image
            }).await;
    
            if let Err(error) = result {
                return Err(Response::internal_server_error(
                    &error.to_string()
                ));
            }
    
            let option = result.unwrap();
            if let None = option {
                return Err(Response::not_found(
                    "Image not found"
                ));
            }
    
            let image = option.unwrap();
    
            images_some.push(image);
        }

        images = Some(images_some);
    }

    let mut video = None;
    if !text_content.video.is_none() {
        let collection = db.collection::<ImageStruct>("image");

        let result = collection.find_one(doc!{
            "uuid": text_content.video.unwrap()
        }).await;

        if let Err(error) = result {
            return Err(Response::internal_server_error(
                &error.to_string()
            ));
        }

        let option = result.unwrap();
        if let None = option {
            return Err(Response::not_found(
                "Video not found"
            ));
        }

        video = Some(option.unwrap());
    }

    Ok(json!({
        "uuid": text_core.uuid,
        "owner": text_core.owner,
        "conversation_id": text_core.conversation_id,
        "text": text_content.text,
        "type": text_core.r#type,
        "images": images,
        "audio": text_content.audio,
        "video": video,
        "attachment": text_content.attachment,
        "seen_by": seen_by,
        "created_at": text_core.created_at,
    }))
}
