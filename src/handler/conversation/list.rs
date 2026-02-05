use futures::StreamExt;
use serde_json::json;
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
    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");

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

        match conversation_core.r#type {
            Conversation::ConversationType::Group => {
                let collection = db.collection::<Conversation::GroupConversationMetadata>("group_conversation_metadata");

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
                    return Ok(Response::not_found("Group conversation metadata not found"));
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
                    return Ok(Response::internal_server_error(&error.to_string()));
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
                let result = collection.find_one(doc!{"uuid": &user_id}).await;

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