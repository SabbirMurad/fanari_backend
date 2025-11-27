use chrono::Utc;
use futures::StreamExt;
use serde_json::json;
use uuid::Uuid;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{web, Error, HttpResponse};
use crate::Middleware::Auth::RequireAccess;
use mongodb::{ClientSession, Database, bson::{Bson, doc}};
use crate::model::{
    AudioStruct,
    Mention,
    Post,
    VideoStruct,
    ImageStruct,
    Account,
    Poll
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqBody {
    page_id: Option<String>,
    caption: Option<String>,
    images: Vec<String>,
    videos: Vec<VideoStruct>,
    audio: Option<AudioStruct>,
    mentions: Vec<Mention>,
    is_nsfw: bool,
    content_warning: Option<String>,
    tags: Vec<String>,
    visibility: Post::PostVisibility,
    poll: Option<PollBody>,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PollBody {
    question: String,
    options: Vec<String>,
    r#type: Poll::PollType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostOwner {
    uuid: String,
    name: String,
    image: Option<ImageStruct>,
    owner_type: Post::PostOwnerType,
    username: String,
    is_me: bool,
    following: bool,
    friend: bool,
}


pub async fn task(
    access: RequireAccess,
    form_data: web::Json<ReqBody>
) -> Result<HttpResponse, Error> {
    let user_id = access.user_id;

    if let Err(res) = check_empty_fields(&form_data) {
        return Ok(Response::bad_request(&res));
    }

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    let owner_type;
    let owner;
    if let Some(page_id) = &form_data.page_id {
        if let Err(error) = check_page_authority(
            &db,
            &mut session,
            &page_id,
            &user_id
        ).await {
            return Ok(error);
        }
      
        owner_type = Post::PostOwnerType::Page;
        owner = page_id.clone();
    }
    else {
        owner_type = Post::PostOwnerType::User;
        owner = user_id.clone();
    }

    let post_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp_millis();
    
    let mut poll_id = None;
    if let Some(poll) = &form_data.poll {
        let uuid = Uuid::new_v4().to_string();
        let collection = db.collection::<Poll::Poll>("poll");
        let result = collection.insert_one(
            &Poll::Poll {
                uuid: uuid.clone(),
                question: poll.question.clone(),
                options: poll.options.clone(),
                r#type: poll.r#type.clone(),
            }
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }
        
        poll_id = Some(uuid);
    }

    let post_core = Post::PostCore {
        uuid: post_id.clone(),
        owner: owner.clone(),
        owner_type: owner_type.clone(),
        caption: form_data.caption.clone(),
        images: form_data.images.clone(),
        videos: form_data.videos.clone(),
        audio: form_data.audio.clone(),
        mentions: form_data.mentions.clone(),
        poll: poll_id,
        tags: form_data.tags.clone(),
        visibility: form_data.visibility.clone(),
        is_nsfw: form_data.is_nsfw.clone(),
        content_warning: form_data.content_warning.clone(),
        modified_at: now,
        created_at: now,
        deleted_at: None,
        suspended_at: None,
        suspended_by: None,
    };
    
    let collection = db.collection::<Post::PostCore>("post_core");
    let result = collection.insert_one(
        &post_core,
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let collection = db.collection::<Post::PostStat>("post_stat");
    let post_stat = Post::PostStat {
        uuid: post_id.clone(),
        comment_count: 0,
        like_count: 0,
        modified_at: now,
        share_count: 0,
        view_count: 0,  
    };

    let result = collection.insert_one(
        &post_stat,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    // Getting The images
    let filter = doc! {
        "uuid": {
            "$in": post_core.images.iter().map(|s| Bson::String(s.clone())).collect::<Vec<Bson>> ()
        }
    };

    let collection = db.collection::<ImageStruct>("image");
    let result = collection.find(filter).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();
    let mut images = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }
        
        let image = result.unwrap();

        let result = collection.update_one(
            doc!{"uuid": &image.uuid},
            doc!{"$set":{"temporary": false}},
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }
        
        images.push(image);
    }

    // getting account info
    let collection = db.collection::<Account::AccountCore>("account_core");
    let result = collection.find_one(doc!{"uuid": &user_id}).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }
    
    let option = result.unwrap();

    if let None = option {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("user not found"));
    }

    let account_core = option.unwrap();

    let collection = db.collection::<Account::AccountProfile>("account_profile");
    let result = collection.find_one(doc!{"uuid": &user_id}).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }
    
    let option = result.unwrap();

    if let None = option {
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::not_found("user not found"));
    }

    let account_profile = option.unwrap();

    let post_owner = PostOwner {
        uuid: user_id.clone(),
        name: format!("{} {}", account_profile.first_name.clone(), account_profile.last_name.clone()),
        image: account_profile.profile_picture.clone(),
        owner_type: Post::PostOwnerType::User,
        username: account_core.username.clone(),
        is_me: true,
        following: false,
        friend: false
    };

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "core": json!({
                "uuid": &post_core.uuid,
                "caption": &post_core.caption,
                "images": &images,
                "videos": &post_core.videos,
                "audio": &post_core.audio,
                "mentions": &post_core.mentions,
                "created_at": &post_core.created_at,
            }),
            "stat": &post_stat,
            "meta": json!({
                "bookmarked": false,
                "liked": false,
            }),
            "owner": serde_json::to_value(&post_owner).unwrap()
        }))
    )
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PageStruct {
  uuid: String,
  owner: String,
  admins: Vec<String>,
}

async fn check_page_authority(
    db: &Database,
    session: &mut ClientSession,
    page_id: &str,
    user_id: &str
) -> Result<(), HttpResponse> {
    let collection = db.collection::<PageStruct>("page");
    let result = collection.find_one(
        doc!{"uuid": page_id},
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        session.abort_transaction().await.ok().unwrap();
        return Err(Response::not_found("page not found"));
    }

    let page = option.unwrap();
    if page.owner != user_id && !page.admins.contains(&user_id.to_string()) {
        session.abort_transaction().await.ok().unwrap();
        return Err(Response::forbidden(
            "You don't have permission to post on this page"
        ));
    }

    Ok(())
}

fn check_empty_fields(data: &ReqBody) -> Result<(), String> {
    if data.images.len() == 0 && data.caption.is_none() && data.videos.len() == 0  && data.audio.is_none() && data.poll.is_none() {
        Err("Nothing to post here".to_string())
    }
    else {
        Ok(())
    }
}