use crate::model::{ImageStruct, Post, account, post::PostOwnerType};
use futures::StreamExt;
use serde_json::Map;
use mongodb::bson::{Bson, doc};
use crate::builtins::mongo::MongoDB;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{ web, Error, HttpResponse};
use crate::Middleware::Auth::RequireAccess;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Query {
    uuid: Option<String>,
    owner: Option<String>,
    fields: Option<String>,
    owner_type: Option<Post::PostOwnerType>,
    visibility: Option<Post::PostVisibility>,
    is_nsfw: Option<bool>,
    limit: i64,
    page: i64,
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
    query: web::Query<Query>
) -> Result<HttpResponse, Error> {
    let user_id = access.user_id;

    let db = MongoDB.connect();

    let mut filter = doc!{};

    if let Some(uuid) = query.uuid.clone() {
        filter.insert("uuid", uuid);
    }
    if let Some(owner) = query.owner.clone() {
        filter.insert("owner", owner);
    }
    if let Some(owner_type) = query.owner_type.clone() {
        filter.insert("owner_type", owner_type.to_string());
    }
    if let Some(visibility) = query.visibility.clone() {
        filter.insert("visibility", visibility.to_string());
    }
    if let Some(is_nsfw) = query.is_nsfw.clone() {
        filter.insert("is_nsfw", is_nsfw);
    }

    let collection = db.collection::<Post::PostCore>("post_core");
    
    let mut cursor = collection.find(
        filter,
    ).sort(doc! { "created_at": -1 })
    .limit(query.limit)
    .skip((query.limit * (query.page - 1)) as u64).await.unwrap();

    let mut posts = Vec::new();
    let mut owner_map = Map::new();
    while let  Some(result) = cursor.next().await {
        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let mut response = Map::new();
        let post_core = result.unwrap();

        let collection = db.collection::<Post::PostStat>("post_stat");
        let result = collection.find_one(
            doc!{"uuid": post_core.uuid.clone()}
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let option = result.unwrap();
        if let None = option {
            return Ok(Response::not_found("Post stat found"));
        }

        let post_stat = option.unwrap();

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
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let mut cursor = result.unwrap();
        let mut images = Vec::new();
        while let Some(result) = cursor.next().await {
            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(&error.to_string()));
            }

            let image = result.unwrap();
            images.push(image);
        }

        response.insert(
            "core".to_string(),
            serde_json::json!({
                "uuid": &post_core.uuid,
                "caption": &post_core.caption,
                "images": &images,
                "mentions": &post_core.mentions,
                "videos": &post_core.videos,
                "audio": &post_core.audio,
                "created_at": &post_core.created_at,
            }),
        );

        response.insert(
            "stat".to_string(),
            serde_json::to_value(
                post_stat
            ).unwrap()
        );

        // Check if post liked
        let collection = db.collection::<Post::PostLike>("post_like");
        let result = collection.count_documents(
            doc!{
                "post_id": post_core.uuid.clone(),
                "liked_by": user_id.clone(),
            }
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let liked = match result.unwrap() {
            0 => false,
            _ => true,
        };

        // Check if post bookmarked
        let collection = db.collection::<Post::PostBookmark>("post_bookmark");
        let result = collection.count_documents(
            doc!{
                "post_id": post_core.uuid.clone(),
                "bookmarked_by": user_id.clone(),
            }
        ).await;


        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let bookmarked = match result.unwrap() {
            0 => false,
            _ => true,
        };

        response.insert(
            "meta".to_string(),
            serde_json::json!({"liked": liked, "bookmarked": bookmarked})
        );

        let owner_id = post_core.owner.clone();
        if let Some(owner) = owner_map.get(&owner_id) {
            response.insert("owner".to_string(), owner.clone());
        }
        else {
            let collection = db.collection::<account::AccountCore>("account_core");

            let result = collection.find_one(
                doc!{"uuid": &owner_id}
            ).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(&error.to_string()));
            }

            let option = result.unwrap();
            if let None = option {
                return Ok(Response::not_found("Account core not found"));
            }

            let account_core = option.unwrap();

            let collection = db.collection::<account::AccountProfile>("account_profile");

            let result = collection.find_one(
                doc!{"uuid": &owner_id}
            ).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(&error.to_string()));
            }

            let option = result.unwrap();
            if let None = option {
                return Ok(Response::not_found("Account profile not found"));
            }

            let account_profile = option.unwrap();

            let post_owner = PostOwner {
                uuid: owner_id.clone(),
                name: format!("{} {}", account_profile.first_name.clone(), account_profile.last_name.clone()),
                image: account_profile.profile_picture.clone(),
                owner_type: PostOwnerType::User,
                username: account_core.username.clone(),
                is_me: false,
                following: false,
                friend: false
            };

            owner_map.insert(
                owner_id.clone(),
                serde_json::to_value(&post_owner).unwrap()
            );

            response.insert(
                "owner".to_string(),
                serde_json::to_value(&post_owner).unwrap()
            );
        }

        posts.push(response);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(posts)
    )
}