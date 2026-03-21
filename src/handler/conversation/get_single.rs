use futures::StreamExt;
use mongodb::Database;
use serde_json::{json, Value};
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{
    Conversation,
    Account,
    ImageStruct,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParticipantQuery {
    conversation_id: String,
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageUuidQuery {
    uuid: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageReadQuery {
    message_id: String,
}

pub async fn task(req: HttpRequest, conversation_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let db = MongoDB.connect();
    let collection = db.collection::<ParticipantQuery>("conversation_participant");

    let result = collection.find(doc!{
        "user_id": &user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();

    let mut conversation_ids: Vec<String> = Vec::new();

    while let Some(participant) = cursor.next().await {
        if let Ok(participant) = participant {
            conversation_ids.push(participant.conversation_id);
        }
    }

    let collection = db.collection::<Conversation::ConversationCore>("conversation_core");
    
    let result = collection.find_one(doc!{
        "uuid":  conversation_id.clone()
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Ok(Response::not_found("Conversation not found"));
    }

    let conversation_core = option.unwrap();

    let single_metadata = match conversation_core.r#type {
        Conversation::ConversationType::Group => {
            return Ok(Response::bad_request("Not a single conversation"));
        },
        Conversation::ConversationType::Single => {
            let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");

            let result = collection.find_one(doc!{
                "conversation_id": &conversation_core.uuid,
                "user_id": {
                    "$ne": &user_id
                }
            }).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(
                    &error.to_string()
                ));
            }

            let option = result.unwrap();
            if let None = option {
                return Ok(Response::not_found(
                    "Conversation participant not found"
                ));
            }

            let conversation_participant = option.unwrap();
            
            match get_single_metadata(
                &db,
                &user_id,
                &conversation_participant.user_id
            ).await {
                Ok(metadata) => metadata,
                Err(error) => {
                    return Ok(error);
                }
            }
        }
    };

    // Check if this conversation is favorited by the current user
    let fav_collection = db.collection::<Conversation::ConversationFavorite>("conversation_favorite");
    let fav_result = fav_collection.find_one(doc!{
        "conversation_id": &conversation_core.uuid,
        "user_id": &user_id
    }).await;

    let is_favorite = match fav_result {
        Ok(option) => option.is_some(),
        Err(error) => {
            log::error!("{:?}", error);
            false
        }
    };

    // Check if this conversation is muted by the current user
    let mute_collection = db.collection::<Conversation::ConversationMuted>("conversation_muted");
    let mute_result = mute_collection.find_one(doc!{
        "conversation_id": &conversation_core.uuid,
        "user_id": &user_id
    }).await;

    let is_muted = match mute_result {
        Ok(option) => option.is_some(),
            Err(error) => {
            log::error!("{:?}", error);
            false
        }
    };

    let common_metadata = json!({
        "is_favorite": is_favorite,
        "is_muted": is_muted
    });

    // Fetch last message content
    let last_text = match conversation_core.last_message_id.clone() {
        Some(last_msg_id) => {
            let result = get_last_text(&db, &last_msg_id).await;
            match result {
                Ok(text) => Some(text),
                Err(error) => {
                    return Ok(error);
                }
            }
        },
        None => None
    };

    // Count unread messages (messages from others that user hasn't read)
    let msg_collection = db.collection::<MessageUuidQuery>("message_core");
    let msg_cursor = msg_collection.find(doc!{
        "conversation_id": &conversation_core.uuid,
        "owner": { "$ne": &user_id }
    }).await;

    let unread_count = match msg_cursor {
        Ok(mut cursor) => {
            let mut other_msg_ids: Vec<String> = Vec::new();
            while let Some(msg) = cursor.next().await {
                if let Ok(msg) = msg {
                    other_msg_ids.push(msg.uuid);
                }
            }

            if other_msg_ids.is_empty() {
                0
            } else {
                let read_collection = db.collection::<MessageReadQuery>("message_read");
                let read_count = read_collection.count_documents(doc!{
                    "message_id": { "$in": &other_msg_ids },
                    "user_id": &user_id
                }).await.unwrap_or(0);

                (other_msg_ids.len() as u64) - read_count
            }
        },
        Err(error) => {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }
    };

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "core": conversation_core,
            "common_metadata": common_metadata,
            "last_text": last_text,
            "unread_count": unread_count,
            "single_metadata": single_metadata,
        }))
    )
}

async fn get_single_metadata(db: &Database, my_id: &str, user_id: &str) -> Result<Value, HttpResponse> {
    let collection = db.collection::<Account::AccountProfile>("account_profile");

    let result = collection.find_one(doc!{
        // "uuid": &conversation_participant.user_id
        "uuid": user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(
            &error.to_string()
        ));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("Account profile not found"));
    }

    let account_profile = option.unwrap();

    let image = match account_profile.profile_picture {
        Some(image) => {
            let collection = db.collection::<ImageStruct>("image");
            let result = collection.find_one(doc!{
                "uuid": image
            }).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Err(Response::internal_server_error(
                    &error.to_string()
                ));
            }

            let option = result.unwrap();
            if let None = option {
                return Err(Response::not_found("Image not found"));
            }

            let image = option.unwrap();
            Some(image)
        },
        None => None
    };

    // Getting account status
    let collection = db.collection::<Account::AccountStatus>("account_status");
    let result = collection.find_one(
        doc!{"uuid": user_id}
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(
            &error.to_string())
        );
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("Account status not found"));
    }

    let account_status = option.unwrap();

    // Check if user is blocked by me
    let collection = db.collection::<Conversation::ConversationBlock>("conversation_block");
    let result = collection.count_documents(doc!{
        "blocker_id": my_id,
        "blocked_id": user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(
            &error.to_string()
        ));
    }

    let is_blocked = result.unwrap() > 0;

    // Check if I am blocked by user
    let result = collection.count_documents(doc!{
        "blocker_id": user_id,
        "blocked_id": my_id
    }).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(
            &error.to_string()
        ));
    }

    let am_blocked = result.unwrap() > 0;

    Ok(
        json!({
            "user_id": account_profile.uuid,
            "first_name": account_profile.first_name,
            "last_name": account_profile.last_name,
            "image": image,
            "online": account_status.online,
            "last_seen": account_status.last_seen,
            "is_blocked": is_blocked,
            "am_blocked": am_blocked
        })
    )
}

async fn get_last_text(db: &Database, last_msg_id: &str) -> Result<Value, HttpResponse> {
    let collection = db.collection::<Conversation::MessageContent>("message_content");

    let result = collection.find_one(doc!{
        "message_id": last_msg_id
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

    let collection = db.collection::<Conversation::MessageCore>("message_core");

    let result = collection.find_one(doc!{
        "uuid": last_msg_id
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

    let text_core = option.unwrap();


    let collection = db.collection::<Conversation::MessageRead>("message_read");

    let result = collection.find(doc!{
        "message_id": last_msg_id
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
        "uuid": text_content.message_id,
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