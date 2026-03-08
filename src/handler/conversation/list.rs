use futures::StreamExt;
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
pub struct ReqQuery {
    limit: Option<u32>,
    offset: Option<u32>,
    ascending: Option<bool>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParticipantQuery {
    conversation_id: String,
    user_id: String,
}

pub async fn task(req: HttpRequest, req_query: web::Query<ReqQuery>) -> Result<HttpResponse, Error> {
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
    
    let limit = req_query.limit.unwrap_or(10) as i64;
    let offset = req_query.offset.unwrap_or(0) as i64;

    let result = collection.find(doc!{
        "uuid": {
            "$in": conversation_ids
        }
    }).sort(doc! {
        "last_message_at": -1,
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

    while let Some(conversation_core) = cursor.next().await {
        if let Err(error) = conversation_core {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let conversation_core = conversation_core.unwrap();

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

        // Fetch last message content
        let last_text = match conversation_core.last_message_id.clone() {
            Some(last_msg_id) => {
                let result = get_last_text(&last_msg_id).await;
                match result {
                    Ok(text) => Some(text),
                    Err(error) => {
                        return Ok(error);
                    }
                }
            },
            None => None
        };

        let common_metadata = json!({
            "is_favorite": is_favorite,
            "is_muted": is_muted
        });

        match conversation_core.r#type {
            Conversation::ConversationType::Group => {
                let collection = db.collection::<Conversation::GroupConversationMetadata>("conversation_group_metadata");

                let result = collection.find_one(doc!{
                    "conversation_id": &conversation_core.uuid
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
                        "Group conversation metadata not found"
                    ));
                }

                let group_conversation_metadata = option.unwrap();

                let image = match group_conversation_metadata.image {
                    Some(image) => {
                        let collection = db.collection::<ImageStruct>("image");

                        let result = collection.find_one(doc!{
                            "uuid": image
                        }).await;

                        if let Err(error) = result {
                            log::error!("{:?}", error);
                            return Ok(Response::internal_server_error(
                                &error.to_string()
                            ));
                        }

                        let option = result.unwrap();
                        if let None = option {
                            return Ok(Response::not_found("Image not found"));
                        }

                        let image = option.unwrap();

                        Some(image)
                    },
                    None => None
                };

                response.push(json!({
                    "core": conversation_core,
                    "common_metadata": common_metadata,
                    "last_text": last_text,
                    "group_metadata": json!({
                        "name": group_conversation_metadata.name,
                        "image": image,
                    })
                }));
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

                let collection = db.collection::<Account::AccountProfile>("account_profile");

                let result = collection.find_one(doc!{
                    "uuid": &conversation_participant.user_id
                }).await;

                if let Err(error) = result {
                    log::error!("{:?}", error);
                    return Ok(Response::internal_server_error(
                        &error.to_string()
                    ));
                }

                let option = result.unwrap();
                if let None = option {
                    return Ok(Response::not_found("Account profile not found"));
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
                            return Ok(Response::internal_server_error(
                                &error.to_string()
                            ));
                        }

                        let option = result.unwrap();
                        if let None = option {
                            return Ok(Response::not_found("Image not found"));
                        }

                        let image = option.unwrap();

                        Some(image)
                    },
                    None => None
                };

                // Getting account status
                let collection = db.collection::<Account::AccountStatus>("account_status");
                let result = collection.find_one(
                    doc!{"uuid": &conversation_participant.user_id}
                ).await;

                if let Err(error) = result {
                    log::error!("{:?}", error);
                    return Ok(Response::internal_server_error(
                        &error.to_string())
                    );
                }

                let option = result.unwrap();
                if let None = option {
                    return Ok(Response::not_found("Account status not found"));
                }

                let account_status = option.unwrap();

                response.push(json!({
                    "core": conversation_core,
                    "common_metadata": common_metadata,
                    "last_text": last_text,
                    "single_metadata": json!({
                        "user_id": account_profile.uuid,
                        "first_name": account_profile.first_name,
                        "last_name": account_profile.last_name,
                        "image": image,
                        "online": account_status.online,
                        "last_seen": account_status.last_seen
                    })
                }));
            }
        }
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(response)
    )
}

async fn get_last_text(last_msg_id: &str) -> Result<Value, HttpResponse> {
    let db = MongoDB.connect();

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