use serde_json::json;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use actix_web::{Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::model::{
    Account::{
        AccountCore,
        AccountProfile,
        AccountSocial,
    },
    ImageStruct,
};

pub async fn task(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let db = MongoDB.connect();

    // Getting core
    let collection = db.collection::<AccountCore>("account_core");
    let result = collection.find_one(doc!{"uuid": &user_id}).await;

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
    let result = collection.find_one(doc!{"uuid": &user_id}).await;

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
    let result = collection.find_one(doc!{"uuid": &user_id}).await;

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
                return Ok(Response::internal_server_error(&error.to_string()));
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

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({
            "core": json!({
                "uuid": &account_core.uuid,
                "username": &account_core.username,
                "role": &account_core.role,
                "two_a_factor_auth_enabled": &account_core.two_a_factor_auth_enabled,
                "two_a_factor_auth_updated": &account_core.two_a_factor_auth_updated,
            }),
            "profile": json!({
                "first_name": &account_profile.first_name,
                "last_name": &account_profile.last_name,
                "phone_number": &account_profile.phone_number,
                "biography": &account_profile.biography,
                "profile_picture": profile_picture,
                "gender": &account_profile.gender,
                "date_of_birth": &account_profile.date_of_birth,
                "profile_verified": &account_profile.profile_verified,
            }),
            "social": json!({
                "like_count": &account_social.like_count,
                "follower_count": &account_social.follower_count,
                "following_count": &account_social.following_count,
                "friend_count": &account_social.friend_count,
                "blocked_count": &account_social.blocked_count,
            }),
        }))
    )
}