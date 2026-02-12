use chrono::Utc;
use mongodb::bson::doc;
use crate::Model::Metadata;
use crate::builtins::mongo::MongoDB;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PostData {
    name: String,
    current_version_android: i64,
    last_supported_version_android: i64,
    emoji_pack_version: i64,
    under_maintenance: bool,
    description: String,
    developer: String,
    developer_email: String,
    developer_phone_number: String,
}

pub async fn task(req: HttpRequest, form_data: web::Json<PostData>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let post_data = sanitize(&form_data);

    if let Err(error) = check_empty_fields(&post_data) {
        return Ok(Response::bad_request(&error));
    }

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;

    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let collection = db.collection::<Metadata::AppMetadata>("app_metadata");

    let result = collection.find_one(
        doc!{},
    ).await;
  
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    let option = result.unwrap();
    if let Some(_) = option {
        let result = collection.update_one(
            doc!{},
            doc!{"$set":{
                "name": &form_data.name,
                "current_version_android": &form_data.current_version_android,
                "last_supported_version_android": &form_data.last_supported_version_android,
                "emoji_pack_version": &form_data.emoji_pack_version,
                "under_maintenance": &form_data.under_maintenance,
                "description": &form_data.description,
                "developer": &form_data.developer,
                "developer_email": &form_data.developer_email,
                "developer_phone_number": &form_data.developer_phone_number,
                "updated_at": Utc::now().timestamp_millis(),
                "updated_by": user_id
            }},
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }
    }
    else {
        let result = collection.insert_one(
            doc!{
                "name": &form_data.name,
                "current_version_android": &form_data.current_version_android,
                "last_supported_version_android": &form_data.last_supported_version_android,
                "emoji_pack_version": &form_data.emoji_pack_version,
                "under_maintenance": &form_data.under_maintenance,
                "description": &form_data.description,
                "developer": &form_data.developer,
                "developer_email": &form_data.developer_email,
                "developer_phone_number": &form_data.developer_phone_number,
                "created_at": Utc::now().timestamp_millis(),
                "created_by": user_id
            },
        ).await;

        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.ok().unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }

    }

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    Ok(HttpResponse::Ok().content_type("application/json").json(
        Response { message: "Successfully Updated".to_string() }
    ))
}

fn sanitize(post_data: &PostData) -> PostData {
  let mut form = post_data.clone();
  form.name = form.name.trim().to_string();
  form.description = form.description.trim().to_string();
  form.developer = form.developer.trim().to_string();
  form.developer_email = form.developer_email.trim().to_string();
  form.developer_phone_number = form.developer_phone_number.trim().to_string();

  form
}

fn check_empty_fields(post_data: &PostData) -> Result<(), String> {
  if post_data.name.len() == 0 {
    Err("name required".to_string())
  }
  else if post_data.description.len() == 0 {
    return Err("description required".to_string());
  }
  else if post_data.developer.len() == 0 {
    return Err("developer required".to_string());
  }
  else if post_data.developer_email.len() == 0 {
    return Err("developer_email required".to_string());
  }
  else if post_data.developer_phone_number.len() == 0 {
    return Err("developer_phone_number required".to_string());
  }
  else {
    Ok(())
  }
}