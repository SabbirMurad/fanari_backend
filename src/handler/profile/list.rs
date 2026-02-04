use mongodb::Database;
use serde::{Deserialize, Serialize};
use serde_json::json;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{
    Account::{
        AccountCore,
        AccountProfile,
        AccountSocial,
        Friends,
        AccountFollow,
        AccountLike,
        AccountBlocked,
    },
    ImageStruct,
};

pub async fn task(req: HttpRequest, target_ids: web::Json<Vec<String>>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let db = MongoDB.connect();

    let mut profiles = Vec::new();

    let unique_users = remove_duplicates(target_ids.clone());

    for target_id in unique_users {
        // Getting core
        let collection = db.collection::<AccountCore>("account_core");
        let result = collection.find_one(doc!{"uuid": &target_id}).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let option = result.unwrap();
        if let None = option {
            return Ok(Response::not_found("user not found"));
        }

        let account_core = option.unwrap();

        // Getting profile
        let collection = db.collection::<AccountProfile>("account_profile");
        let result = collection.find_one(doc!{"uuid": &target_id}).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let option = result.unwrap();
        if let None = option {
            return Ok(Response::not_found("user not found"));
        }

        let account_profile = option.unwrap();

        // Getting social
        let collection = db.collection::<AccountSocial>("account_social");
        let result = collection.find_one(doc!{"uuid": &target_id}).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let option = result.unwrap();
        if let None = option {
            return Ok(Response::not_found("user not found"));
        }

        let account_social = option.unwrap();

        let profile_picture: Option<ImageStruct> = match account_profile.profile_picture {
            Some(image_id) => {
                let collection = db.collection::<ImageStruct>("image");
                let result = collection.find_one(doc!{"uuid": &image_id}).await;

                if let Err(error) = result {
                    log::error!("{:?}", error);
                    return Ok(Response::internal_server_error(
                        &error.to_string()
                    ));
                }

                let option = result.unwrap();
                if let None = option {
                    None
                } else {
                    Some(option.unwrap())
                }
            },
            None => None
        };

        let profile_stat = match get_profile_stat(
            &user_id,
            &target_id,
            &db
        ).await {
            Ok(stat) => stat,
            Err(error) => {
                return Ok(error);
            }
        };


        profiles.push(json!({
            "core": json!({
                "uuid": &account_core.uuid,
                "username": &account_core.username,
                "role": &account_core.role,
            }),
            "profile": json!({
                "first_name": &account_profile.first_name,
                "last_name": &account_profile.last_name,
                "biography": &account_profile.biography,
                "profile_picture": profile_picture,
                "gender": &account_profile.gender,
                "profile_verified": &account_profile.profile_verified,
            }),
            "social": json!({
                "like_count": &account_social.like_count,
                "follower_count": &account_social.follower_count,
                "following_count": &account_social.following_count,
                "friend_count": &account_social.friend_count,
            }),
            "stat": profile_stat,
        }));

    }

    
    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(profiles)
    )
}

fn remove_duplicates(items: Vec<String>) -> Vec<String> {
    let mut unique_items: Vec<String> = Vec::new();
    for item in items {
        if !unique_items.contains(&item) {
            unique_items.push(item);
        }
    }

    unique_items
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileStat {
    pub is_friend: bool,
    pub is_following: bool,
    pub is_follower: bool,
    pub is_liked: bool,
    pub is_blocked: bool,
    pub myself: bool
}

async fn get_profile_stat(user_id: &String, target_id: &String, db: &Database) -> Result<ProfileStat, HttpResponse> {
    //Check if is friend
    let is_friend = match user_id == target_id {
        true => false,
        false => {
            let collection = db.collection::<Friends>("friends");
            let result = collection.count_documents(doc!{
                "$or": [
                    {"requested_by": user_id, "accepted_by": target_id},
                    {"requested_by": target_id, "accepted_by": user_id},
                ]
            }).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Err(Response::internal_server_error(&error.to_string()));
            }

            let count = result.unwrap();
            count > 0
        }
    };

    //Check if is following
    let is_following = match user_id == target_id {
        true => false,
        false => {
            let collection = db.collection::<AccountFollow>("follow");
            let result = collection.count_documents(doc!{
                "followed_by": user_id,
                "user_id": target_id
            }).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Err(Response::internal_server_error(&error.to_string()));
            }

            let count = result.unwrap();
            count > 0
        }
    };

    //Check if is follower
    let is_follower = match user_id == target_id {
        true => false,
        false => {
            let collection = db.collection::<AccountFollow>("account_follow");
            let result = collection.count_documents(doc!{
                "followed_by": target_id,
                "user_id": user_id
            }).await;

            if let Err(error) = result {
                log::error!("{:?}", error);
                return Err(Response::internal_server_error(&error.to_string()));
            }

            let count = result.unwrap();
            count > 0
        }
    };

    //Check if is profile liked
    let collection = db.collection::<AccountLike>("account_like");
    let result = collection.count_documents(doc!{
        "user_id": target_id,
        "liked_by": user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let count = result.unwrap();
    let is_liked = count > 0;

    //Check if is blocked
    let collection = db.collection::<AccountBlocked>("account_blocked");
    let result = collection.count_documents(doc!{
        "blocked": target_id,
        "blocked_by": user_id
    }).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let count = result.unwrap();
    let is_blocked = count > 0;

    Ok(ProfileStat {
        is_friend,
        is_following,
        is_follower,
        is_liked,
        is_blocked,
        myself: user_id == target_id
    })
}