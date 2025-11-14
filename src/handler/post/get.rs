use crate::model::Post;
use futures::StreamExt;
use serde_json::Map;
use mongodb::bson::doc;
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

pub async fn task(
    access: RequireAccess,
    query: web::Query<Query>
) -> Result<HttpResponse, Error> {
    let _user_id = access.user_id;

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

        response.insert(
            "core".to_string(),
            serde_json::to_value(
                post_core
            ).unwrap()
        );

        response.insert(
            "stat".to_string(),
            serde_json::to_value(
                post_stat
            ).unwrap()
        );

        posts.push(response);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(posts)
    )
}