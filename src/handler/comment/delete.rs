use chrono::Utc;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::utils::response::Response;
use crate::model::{ Comment, Account::AccountRole };
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::middleware::auth::{require_access, AccessRequirement};

pub async fn task(
    req: HttpRequest,
    comment_id: web::Path<String>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::Role(AccountRole::Administrator)
    )?;

    let user_id = user.user_id;

    let comment_id = comment_id.into_inner();
    if comment_id.len() == 0 {
        return Ok(Response::bad_request("comment id required"));
    }

    let db = MongoDB.connect();

    //finding the post
    let collection = db.collection::<Comment::CommentCore>("comment_core");
    let result = collection.find_one(
        doc!{ "uuid": &comment_id},
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Ok(Response::not_found("comment not found"));
    }

    let post = option.unwrap();
    if post.owner != user_id {
        return Ok(Response::forbidden(
            "You are not authorized to delete this post"
        ));
    }

    let collection = db.collection::<Comment::CommentCore>("comment_core");
    let now = Utc::now().timestamp_millis();
    let result = collection.update_one(
        doc!{"uuid": &comment_id},
        doc!{"$set": {
            "deleted_at": now,
            "modified_at": now,
        }},
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let update_result = result.unwrap();
    if update_result.matched_count == 0 {
        return Ok(Response::not_found("post not found"));
    }

    Ok(HttpResponse::Ok().content_type("application/json").json(
        Response { message: "Successfully Deleted".to_string() }
    ))
}