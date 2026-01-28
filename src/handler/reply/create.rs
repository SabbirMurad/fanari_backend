use uuid::Uuid;
use chrono::Utc;
use serde_json::json;
use mongodb::Database;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
// use crate::Integrations::Firebase;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::middleware::auth::{require_access, AccessRequirement};
use crate::model::{Comment, AudioStruct, Mention, Reply, Account::AccountRole};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqBody {
    comment_id: String,
    text: Option<String>,
    images: Vec<String>,
    audio: Option<AudioStruct>,
    mentions: Vec<Mention>,
}

pub async fn task(
    req: HttpRequest,
    form_data: web::Json<ReqBody>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::Role(AccountRole::Administrator)
    )?;

    let user_id = user.user_id;

    if let Err(res) = check_empty_fields(&form_data) {
        return Ok(Response::bad_request(&res));
    }

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let reply_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();

    // insert comment core
    let collection = db.collection::<Reply::ReplyCore>("reply_core");
    let comment_core = Reply::ReplyCore {
        uuid: reply_id.clone(),
        owner: user_id.clone(),
        comment_id: form_data.comment_id.clone(),
        text: form_data.text.clone(),
        images: form_data.images.clone(),
        audio: form_data.audio.clone(),
        mentions: form_data.mentions.clone(),
        status: Reply::ReplyStatus::Active,
        created_at: now,
        deleted_at: None,
        suspended_at: None,
        suspended_by: None,
        is_edited: false,
        modified_at: now,
    };

    let result = collection.insert_one(comment_core).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    // insert comment stat
    let collection = db.collection::<Reply::ReplyStat>("reply_stat");
    let comment_stat = Reply::ReplyStat {
        uuid: reply_id.clone(),
        modified_at: now,
        like_count: 0,
    };

    let result = collection.insert_one(comment_stat).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    // Update post comment count
    let collection = db.collection::<Comment::CommentStat>("comment_stat");
    let result = collection.update_one(
        doc!{ "uuid": &form_data.comment_id },
        doc!{
            "$inc":{ "reply_count": 1 },
            "$set":{ "modified_at": now }
        },
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let count = result.unwrap().modified_count;
    if count == 0 {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("post not found"));
    }

    //Finding the post owner id
    let post_owner = match get_comment_owner(&db, &form_data.comment_id).await {
        Ok(post_owner) => post_owner,
        Err(error) => {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(error);
        },
    };

    if post_owner != user_id {
        
    }

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "uuid": &reply_id
        }))
    )
}

async fn get_comment_owner(
    db: &Database,
    comment_id: &str
) -> Result<String, HttpResponse> {
    #[derive(Debug, Deserialize, Serialize)]
    struct CommentCoreRead {
        uuid: String,
        owner: String,
    }

    let collection = db.collection::<CommentCoreRead>("comment_core");
    let result = collection.find_one(
        doc!{ "uuid": comment_id },
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("post not found"));
    }

    let post = option.unwrap();
    let post_owner = post.owner;

    Ok(post_owner)
}

fn check_empty_fields(data: &ReqBody) -> Result<(), String> {
    if data.images.len() == 0 && data.text.is_none() && data.audio.is_none() {
        Err("Nothing to comment here".to_string())
    }
    else if data.comment_id.len() == 0 {
        Err("Comment id required".to_string())
    }
    else {
        Ok(())
    }
}