use crate::utils::string;
use crate::model::Post;
use serde_json::{ Map, Value};
use mongodb::{bson::doc, Database};
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
    offset: i64,
}

pub async fn task(
    access: RequireAccess,
    query: web::Query<Query>
) -> Result<HttpResponse, Error> {
    let user_id = access.user_id;

    let mut response = Map::new();

    let db = MongoDB.connect();
    if let Some(fields) = query.fields.clone() {
        let fields = match string::parse_comma_separated(&fields) {
            Ok(fields) => fields,
            Err(error) => return Ok(Response::bad_request(&error)),
        };

        for field in fields {
            if let Ok(value) = string::strip_prefix("core", field) {
                let sub_fields = match value.len() {
                    0 => vec![],
                    _ => match string::parse_comma_separated(&value) {
                        Ok(sub_fields) => sub_fields,
                        Err(error) => {
                            return Ok(Response::bad_request(&error));
                        },
                    },
                };
                
                let post_core = match get_post_core(
                    &user_id,
                    &db,
                    sub_fields
                ).await {
                    Ok(post_core) => post_core,
                    Err(error) => return Ok(error),
                };

                response.insert("core".to_string(), post_core);
            }
            else if let Ok(value) = string::strip_prefix("stat", field) {
                let sub_fields = match value.len() {
                    0 => vec![],
                    _ => match string::parse_comma_separated(&value) {
                        Ok(sub_fields) => sub_fields,
                        Err(error) => {
                            return Ok(Response::bad_request(&error));
                        },
                    },
                };
                
                let post_stat = match get_post_stat(
                    &user_id,
                    &db,
                    sub_fields
                ).await {
                    Ok(post_stat) => post_stat,
                    Err(error) => return Ok(error),
                };

                response.insert("stat".to_string(), post_stat);
            }
            else {
                return Ok(Response::bad_request(
                    &format!("Invalid field: {}", field)
                ));
            }
        }
    }
    else {
        let post_core = match get_post_core(
            &user_id,
            &db,
            vec![]
        ).await {
            Ok(post_core) => post_core,
            Err(error) => return Ok(error),
        };

        response.insert("core".to_string(), post_core);

        let post_stat = match get_post_stat(
            &user_id,
            &db,
            vec![]
        ).await {
            Ok(post_stat) => post_stat,
            Err(error) => return Ok(error),
        };

        response.insert("stat".to_string(), post_stat);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(Value::Object(response))
    )
}

async fn get_post_core(
    user_id: &str,
    db: &Database,
    sub_fields: Vec<&str>
) -> Result<serde_json::Value, HttpResponse> {
    let collection = db.collection::<Post::PostCore>("post_core");
    let result = collection.find_one(doc!{"uuid": user_id}).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("account not found"));
    }

    if sub_fields.len() > 0 {
        let post_core = option.unwrap();
        let mut response = Map::new();
        for field in sub_fields {
            match field {
                "owner" => response.insert(
                    "owner".to_string(),
                    Value::String(post_core.owner.clone())
                ),
                "caption" => response.insert(
                    "caption".to_string(),
                    if post_core.caption.is_none() {
                        Value::Null
                    }
                    else {
                        Value::String(post_core.caption.clone().unwrap())
                    }
                ),
                "images" => response.insert(
                    "images".to_string(),
                    serde_json::to_value(
                        post_core.images.clone()
                    ).unwrap()
                ),
                "videos" => response.insert(
                    "videos".to_string(),
                    serde_json::to_value(
                        post_core.videos.clone()
                    ).unwrap()
                ),
                "audio" => response.insert(
                    "audio".to_string(),
                    if post_core.audio.is_none() {
                        Value::Null
                    }
                    else {
                        serde_json::to_value(
                            post_core.audio.clone().unwrap()
                        ).unwrap()
                    }
                ),
                "mentions" => response.insert(
                    "mentions".to_string(),
                    serde_json::to_value(
                        post_core.mentions.clone()
                    ).unwrap()
                ),
                "owner_type" => response.insert(
                    "owner_type".to_string(),
                    Value::String(
                        post_core.owner_type.clone().to_string()
                    )
                ),
                "visibility" => response.insert(
                    "visibility".to_string(),
                    Value::String(
                        post_core.visibility.clone().to_string()
                    )
                ),
                "tags" => response.insert(
                    "tags".to_string(),
                    serde_json::to_value(
                        post_core.tags.clone()
                    ).unwrap()
                ),
                "is_nsfw" => response.insert(
                    "is_nsfw".to_string(),
                    Value::Bool(
                        post_core.is_nsfw.clone()
                    )
                ),
                "content_warning" => response.insert(
                    "content_warning".to_string(),
                    if post_core.content_warning.is_none() {
                        Value::Null
                    }
                    else {
                        Value::String(
                            post_core.content_warning.clone().unwrap()
                        )
                    }
                ),
                "created_at" => response.insert(
                    "created_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_core.created_at.clone()
                    ))
                ),
                "modified_at" => response.insert(
                    "modified_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_core.modified_at.clone()
                    ))
                ),
                "suspended_at" => {
                    if post_core.suspended_at.is_none() {
                        response.insert(
                            "suspended_at".to_string(),
                            Value::Null
                        )
                    }
                    else {
                        response.insert(
                            "suspended_at".to_string(),
                            Value::Number(serde_json::Number::from(
                                post_core.suspended_at.clone().unwrap()
                            ))
                        )
                    }
                },
                "suspended_by" => {
                    if post_core.suspended_by.is_none() {
                        response.insert(
                            "suspended_by".to_string(),
                            Value::Null
                        )
                    }
                    else {
                        response.insert(
                            "suspended_by".to_string(),
                            Value::String(post_core.suspended_by.clone().unwrap())
                        )
                    }
                },
                others => return Err(Response::bad_request(
                    &format!("Invalid field: {others}"))
                ),
            };
        }

        return Ok(serde_json::to_value(response).unwrap());
    }

    Ok(serde_json::to_value(option.unwrap()).unwrap())
}

async fn get_post_stat(
    user_id: &str,
    db: &Database,
    sub_fields: Vec<&str>
) -> Result<serde_json::Value, HttpResponse> {
    let collection = db.collection::
    <Post::PostStat>("post_stat");
    let result = collection.find_one(doc!{"uuid": user_id}).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Err(Response::internal_server_error(&error.to_string()));
    }

    let option = result.unwrap();
    if let None = option {
        return Err(Response::not_found("account not found"));
    }

    if sub_fields.len() > 0 {
        let post_stat = option.unwrap();
        let mut response = Map::new();
        for field in sub_fields {
            match field {
                "like_count" => response.insert(
                    "like_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_stat.like_count.clone()
                    ))
                ),
                "comment_count" => response.insert(
                    "comment_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_stat.comment_count.clone()
                    ))
                ),
                "share_count" => response.insert(
                    "share_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_stat.share_count.clone()
                    ))
                ),
                "view_count" => response.insert(
                    "view_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_stat.view_count.clone()
                    ))
                ),
                "modified_at" => response.insert(
                    "modified_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        post_stat.modified_at.clone()
                    ))
                ),
                others => return Err(Response::bad_request(
                    &format!("Invalid field: {others}"))
                ),
            };
        }

        return Ok(serde_json::to_value(response).unwrap());
    }

    Ok(serde_json::to_value(option.unwrap()).unwrap())
}