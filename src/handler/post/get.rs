use futures::StreamExt;
use serde_json::Map;
use mongodb::{Database, bson::{Bson, doc}};
use crate::builtins::mongo::MongoDB;
use crate::utils::response::Response;
use serde::{ Serialize, Deserialize };
use actix_web::{ web, Error, HttpResponse, HttpRequest };
use crate::Middleware::Auth::{require_access, AccessRequirement};
use crate::model::{
    ImageStruct,
    Post,
    Poll,
};

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

pub async fn task(
    req: HttpRequest,
    query: web::Query<Query>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

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
    
    let result = collection.find(
        filter,
    ).sort(doc! { "created_at": -1 })
    .limit(query.limit)
    .skip((query.limit * (query.page - 1)) as u64).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();

    let mut posts = Vec::new();

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
        let images = match get_images(&db, post_core.images.clone()).await {
            Ok(images) => images,
            Err(error) => return Ok(error),
        };

        // Getting The video thumbnails
        let video_thumbnails = match get_images(&db, post_core.videos.clone()).await {
            Ok(images) => images,
            Err(error) => return Ok(error),
        };

        //Getting poll information
        let poll = match get_poll(&db, &post_core.poll.clone()).await {
            Ok(poll) => poll,
            Err(error) => return Ok(error),
        };

        //getting the mentions
        let collection = db.collection::<Post::PostMention>("post_mention");
        let result = collection.find(doc!{
            "post_id": &post_core.uuid
        }).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let mut mention_cursor = result.unwrap();

        let mut mentions = Vec::new();
        
        while let Some(result) = mention_cursor.next().await {
            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(&error.to_string()));
            }

            mentions.push(result.unwrap());
        }

        //getting the tags
        let collection = db.collection::<Post::PostTag>("post_tag");
        let result = collection.find(doc!{
            "post_id": &post_core.uuid
        }).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let mut tag_cursor = result.unwrap();

        let mut tags = Vec::new();
        
        while let Some(result) = tag_cursor.next().await {
            if let Err(error) = result {
                log::error!("{:?}", error);
                return Ok(Response::internal_server_error(&error.to_string()));
            }

            tags.push(result.unwrap().tag);
        }

        response.insert(
            "core".to_string(),
            serde_json::json!({
                "uuid": &post_core.uuid,
                "caption": &post_core.caption,
                "images": &images,
                "tags": &tags,
                "mentions": &mentions,
                "videos": &video_thumbnails,
                "audio": &post_core.audio,
                "poll": &poll,
                "created_at": &post_core.created_at,
                "owner_id": &post_core.owner,
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

        posts.push(response);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(posts)
    )
}

async fn get_poll(db: &Database, poll_id: &Option<String>) -> Result<Option<serde_json::Value>, HttpResponse> {
    if poll_id.is_none() {
        return Ok(None);
    }

    let collection = db.collection::<Poll::Poll>("poll");
    let result = collection.find_one(
        doc!{"uuid": poll_id.clone().unwrap()}
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("Poll not found"));
    }

    let poll = option.unwrap();
    let mut options = Vec::new();
    let mut total_vote = 0;
    for option in poll.options.iter() {
        let collection = db.collection::<Poll::PollVote>("poll_vote");
        let result = collection.count_documents(
            doc!{
                "poll_id": poll.uuid.clone(),
                "option": option.clone(),
            }
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Err(Response::internal_server_error(&error.to_string()));
        }

        let vote = result.unwrap();
        total_vote += vote;

        options.push(serde_json::json!({
            "text": option.clone(),
            "vote": vote
        }));
    }

    let value = serde_json::json!({
        "uuid": &poll.uuid,
        "question": &poll.question,
        "type": &poll.r#type,
        "can_add_option": false,
        "options": &options,
        "total_vote": total_vote,
        "selected_option": []
    });

    Ok(Some(value))
}

// async fn get_post_owner(db: &Database, owner_id: &str) -> Result<PostOwner, HttpResponse> {
//     let collection = db.collection::<Account::AccountCore>("account_core");

//     let result = collection.find_one(
//         doc!{"uuid": &owner_id}
//     ).await;

//     if let Err(error) = result {
//         log::error!("{:?}", error);
//         return Err(Response::internal_server_error(&error.to_string()));
//     }

//     let option = result.unwrap();
//     if let None = option {
//         return Err(Response::not_found("Account core not found"));
//     }

//     let account_core = option.unwrap();

//     let collection = db.collection::<Account::AccountProfile>("account_profile");

//     let result = collection.find_one(
//         doc!{"uuid": &owner_id}
//     ).await;

//     if let Err(error) = result {
//         log::error!("{:?}", error);
//         return Err(Response::internal_server_error(&error.to_string()));
//     }

//     let option = result.unwrap();
//     if let None = option {
//         return Err(Response::not_found("Account profile not found"));
//     }

//     let account_profile = option.unwrap();

//     let profile_picture: Option<ImageStruct> = match account_profile.profile_picture {
//         Some(image_id) => {
//             let collection = db.collection::<ImageStruct>("image");
//             let result = collection.find_one(doc!{"uuid": &image_id}).await;

//             if let Err(error) = result {
//                 log::error!("{:?}", error);
//                 return Err(Response::internal_server_error(&error.to_string()));
//             }

//             let option = result.unwrap();
//             if let None = option {
//                 None
//             } else {
//                 Some(option.unwrap())
//             }
//         },
//         None => None
//     };

//     let post_owner = PostOwner {
//         uuid: owner_id.to_string(),
//         name: format!("{} {}", account_profile.first_name.clone(), account_profile.last_name.clone()),
//         image: profile_picture,
//         owner_type: PostOwnerType::User,
//         username: account_core.username.clone(),
//         is_me: false,
//         following: false,
//         friend: false
//     };

//     Ok(post_owner)
// }

async fn get_images(db: &Database, image_ids: Vec<String>) -> Result<Vec<ImageStruct>, HttpResponse> {
    let mut images = Vec::new();

    if image_ids.len() == 0 {
        return Ok(images);
    }

    let filter = doc! {
        "uuid": {
            "$in": image_ids.iter().map(|s| Bson::String(s.clone())).collect::<Vec<Bson>> ()
        }
    };

    let collection = db.collection::<ImageStruct>("image");
    let result = collection.find(filter).await;
        
    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();
    while let Some(result) = cursor.next().await {
        if let Err(error) = result {
            log::error!("{:?}", error);
            return Err(Response::internal_server_error(&error.to_string()));
        }

        let image = result.unwrap();
        images.push(image);
    }

    Ok(images)
}