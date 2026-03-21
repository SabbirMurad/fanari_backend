use chrono::Utc;
use futures::StreamExt;
use uuid::Uuid;
use serde_json::json;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::handler::web_socket::message::AddToRoom;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use crate::Handler::WebSocket::lobby::Lobby;
use actix::Addr;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{Conversation, ImageStruct, Account};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqBody { other_user: String}

pub async fn task(
    req: HttpRequest,
    req_body: web::Json<ReqBody>,
    srv: web::Data<Addr<Lobby>>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    //Check if conversation between users exist
    if let Some(_) = single_conversation_exists(&db, &user_id, &req_body.other_user).await {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::conflict(
            "conversation already exists between users"
        ));
    }

    //Creating Conversation
    let conversation_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();

    let conversation = Conversation::ConversationCore {
        uuid: conversation_id.clone(),
        last_message_at: now,
        created_at: now,
        r#type: Conversation::ConversationType::Single,
        last_message_id: None,
    };

    let collection = db.collection::<Conversation::ConversationCore>("conversation_core");
    let result = collection.insert_one(
        &conversation,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    //Creating Conversation member 1
    let conversation_participant = Conversation::ConversationParticipant {
        conversation_id: conversation_id.clone(),
        joined_at: now,
        last_message_read_id: None,
        role: Conversation::ConversationRole::Member,
        user_id: user_id.clone(),
    };

    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");
    let result = collection.insert_one(
        &conversation_participant,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    //Creating Conversation member 2
    let conversation_participant = Conversation::ConversationParticipant {
        conversation_id: conversation_id.clone(),
        joined_at: now,
        last_message_read_id: None,
        role: Conversation::ConversationRole::Member,
        user_id: req_body.other_user.clone(),
    };

    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");
    let result = collection.insert_one(
        &conversation_participant,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

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
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Ok(Response::not_found("Account status not found"));
    }

    let account_status = option.unwrap();

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

/* Notify both users to join the new room */
srv.do_send(AddToRoom {
    user_id: user_id.clone(),
    conversation_id: conversation_id.clone(),
});
srv.do_send(AddToRoom {
    user_id: req_body.other_user.clone(),
    conversation_id: conversation_id.clone(),
});

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "core": conversation,
            "single_metadata": json!({
                "user_id": account_profile.uuid,
                "first_name": account_profile.first_name,
                "last_name": account_profile.last_name,
                "image": image,
                "online": account_status.online,
                "last_seen": account_status.last_seen,
                "is_blocked": false,
                "am_blocked": false
            }),
            "common_metadata": json!({
                "is_favorite": false,
                "is_muted": false
            })
        }))
    )
}

pub async fn single_conversation_exists(
    db: &mongodb::Database,
    user_a: &str,
    user_b: &str,
) -> Option<String> {
    let collection = db.collection::<Conversation::ConversationParticipant>("conversation_participant");

    // Find all Single conversation_ids that user_a is part of
    let pipeline = vec![
        // Match conversations where user_a is a participant
        doc! { "$match": { "user_id": user_a } },

        // Lookup to find user_b in the same conversation
        doc! { "$lookup": {
            "from": "conversation_participant",
            "localField": "conversation_id",
            "foreignField": "conversation_id",
            "as": "other_participants"
        }},

        // Filter: other_participants must contain user_b
        doc! { "$match": {
            "other_participants.user_id": user_b
        }},

        // Lookup conversation_core to check type = Single
        doc! { "$lookup": {
            "from": "conversation_core",
            "localField": "conversation_id",
            "foreignField": "uuid",
            "as": "core"
        }},

        // Filter: must be Single type
        doc! { "$match": {
            "core.type": "Single"
        }},

        doc! { "$limit": 1 },
    ];

    let mut cursor = collection.aggregate(pipeline).await.ok()?;

    if let Some(Ok(doc)) = cursor.next().await {
        let conversation_id = doc.get_str("conversation_id").ok()?.to_string();
        return Some(conversation_id);
    }

    None
}