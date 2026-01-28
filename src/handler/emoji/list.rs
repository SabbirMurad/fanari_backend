use futures::StreamExt;
use mongodb::bson::doc;
use crate::BuiltIns::mongo::MongoDB;
use crate::model::Emoji;
// use crate::Integrations::Firebase;
use crate::utils::response::Response;
use actix_web::{Error, HttpResponse, HttpRequest};
use crate::middleware::auth::{require_access, AccessRequirement};

pub async fn task(req: HttpRequest) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let _user_id = user.user_id;

    let db = MongoDB.connect();

    let collection = db.collection::<Emoji>("emoji");
    let result = collection.find(doc!{}).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let mut cursor = result.unwrap();
    let mut emojis = Vec::new();
    while let Some(result) = cursor.next().await {
        if let Err(error) = result {
            log::error!("{:?}", error);
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        let emoji = result.unwrap();
        emojis.push(emoji);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(emojis)
    )
}