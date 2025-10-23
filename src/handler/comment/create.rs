use mongodb::Database;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use mongodb::bson::doc;
use crate::builtins;
use crate::BuiltIns::mongo::MongoDB;
use crate::Integrations::Firebase;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse};
use crate::middleware::auth::RequireAccess;
use crate::model::{Comment, Post, Mention, AudioStruct, ImageStruct};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostData {
    post_id: String,
    text: Option<String>,
    images: Vec<Vec<u8>>,
    audio: Option<AudioStruct>,
    mentions: Vec<Mention>,
}

pub async fn task(
    access: RequireAccess,
    form_data: web::Json<PostData>
) -> Result<HttpResponse, Error> {
    let user_id = access.user_id;

    if let Err(res) = check_empty_fields(form_data.clone()) {
        return Ok(Response::bad_request(&res));
    }

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    let mut images = Vec::new();
    for image in form_data.images.clone() {
        let result = builtins::image::add(
            None,
            image,
            builtins::image::ImageFrom::Comment
        ).await;
    
        if let Err(err) = result {
            log::error!("{:?}", err);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::bad_request(&err));
        }
    
        let image_info = result.unwrap();
        let image_data = ImageStruct {
            uuid: image_info.uuid,
            width: image_info.width,
            height: image_info.height,
            r#type: image_info.r#type,
        };

        images.push(image_data);
    }

    let comment_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();

    // insert comment core
    let collection = db.collection::<Comment::CommentCore>("comment_core");
    let comment_core = Comment::CommentCore {
        uuid: comment_id.clone(),
        owner: user_id.clone(),
        post_id: form_data.post_id.clone(),
        text: form_data.text.clone(),
        images: images.clone(),
        audio: form_data.audio.clone(),
        mentions: form_data.mentions.clone(),
        status: Comment::CommentStatus::Active,
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
    let collection = db.collection::<Comment::CommentStat>("comment_stat");
    let comment_stat = Comment::CommentStat {
        uuid: comment_id.clone(),
        modified_at: now,
        like_count: 0,
        reply_count: 0,
    };

    let result = collection.insert_one(comment_stat).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    // Update post comment count
    let collection = db.collection::<Post::PostStat>("post_stat");
    let result = collection.update_one(
        doc!{ "uuid": &form_data.post_id },
        doc!{
            "$inc":{ "comment_count": 1 },
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
    let post_owner = match get_post_owner(&db, &form_data.post_id).await {
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
            "uuid": &comment_id
        }))
    )
}

async fn get_post_owner(
    db: &Database,
    post_id: &str
) -> Result<String, HttpResponse> {
    #[derive(Debug, Deserialize, Serialize)]
    struct PostCoreRead {
        uuid: String,
        owner: String,
    }

    let collection = db.collection::<PostCoreRead>("post_core");
    let result = collection.find_one(
        doc!{ "uuid": post_id },
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

fn check_empty_fields(data: PostData) -> Result<(), String> {
    if data.images.len() == 0 && data.text.is_none() && data.audio.is_none() {
        Err("Nothing to comment here".to_string())
    }
    else if data.post_id.len() == 0 {
        Err("Post id required".to_string())
    }
    else {
        Ok(())
    }
}