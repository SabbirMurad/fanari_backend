use crate::utils::string;
use crate::model::Comment;
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
    post_id: Option<String>,
    fields: Option<String>,
    status: Option<Comment::CommentStatus>,
    is_edited: Option<bool>,
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
                
                let comment_core = match get_comment_core(
                    &user_id,
                    &db,
                    sub_fields
                ).await {
                    Ok(comment_core) => comment_core,
                    Err(error) => return Ok(error),
                };

                response.insert("core".to_string(), comment_core);
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
                
                let comment_stat = match get_comment_stat(
                    &user_id,
                    &db,
                    sub_fields
                ).await {
                    Ok(comment_stat) => comment_stat,
                    Err(error) => return Ok(error),
                };

                response.insert("stat".to_string(), comment_stat);
            }
            else {
                return Ok(Response::bad_request(
                    &format!("Invalid field: {}", field)
                ));
            }
        }
    }
    else {
        let comment_core = match get_comment_core(
            &user_id,
            &db,
            vec![]
        ).await {
            Ok(comment_core) => comment_core,
            Err(error) => return Ok(error),
        };

        response.insert("core".to_string(), comment_core);

        let comment_stat = match get_comment_stat(
            &user_id,
            &db,
            vec![]
        ).await {
            Ok(comment_stat) => comment_stat,
            Err(error) => return Ok(error),
        };

        response.insert("stat".to_string(), comment_stat);
    }

    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(Value::Object(response))
    )
}

async fn get_comment_core(
    user_id: &str,
    db: &Database,
    sub_fields: Vec<&str>
) -> Result<serde_json::Value, HttpResponse> {
    let collection = db.collection::<Comment::CommentCore>("comment_core");
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
        let comment_core = option.unwrap();
        let mut response = Map::new();
        for field in sub_fields {
            match field {
                "owner" => response.insert(
                    "owner".to_string(),
                    Value::String(comment_core.owner.clone())
                ),
                "post_id" => response.insert(
                    "post_id".to_string(),
                    Value::String(comment_core.post_id.clone())
                ),
                "text" => response.insert(
                    "text".to_string(),
                    if comment_core.text.is_none() {
                        Value::Null
                    }
                    else {
                        Value::String(comment_core.text.clone().unwrap())
                    }
                ),
                "images" => response.insert(
                    "images".to_string(),
                    serde_json::to_value(
                        comment_core.images.clone()
                    ).unwrap()
                ),
                "audio" => response.insert(
                    "audio".to_string(),
                    if comment_core.audio.is_none() {
                        Value::Null
                    }
                    else {
                        serde_json::to_value(
                            comment_core.audio.clone().unwrap()
                        ).unwrap()
                    }
                ),
                "status" => response.insert(
                    "status".to_string(),
                    Value::String(
                        comment_core.status.clone().to_string()
                    )
                ),
                "is_edited" => response.insert(
                    "is_edited".to_string(),
                    Value::Bool(
                        comment_core.is_edited.clone()
                    )
                ),
                "mentions" => response.insert(
                    "mentions".to_string(),
                    serde_json::to_value(
                        comment_core.mentions.clone()
                    ).unwrap()
                ),
                "created_at" => response.insert(
                    "created_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        comment_core.created_at.clone()
                    ))
                ),
                "modified_at" => response.insert(
                    "modified_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        comment_core.modified_at.clone()
                    ))
                ),
                "suspended_at" => {
                    if comment_core.suspended_at.is_none() {
                        response.insert(
                            "suspended_at".to_string(),
                            Value::Null
                        )
                    }
                    else {
                        response.insert(
                            "suspended_at".to_string(),
                            Value::Number(serde_json::Number::from(
                                comment_core.suspended_at.clone().unwrap()
                            ))
                        )
                    }
                },
                "suspended_by" => {
                    if comment_core.suspended_by.is_none() {
                        response.insert(
                            "suspended_by".to_string(),
                            Value::Null
                        )
                    }
                    else {
                        response.insert(
                            "suspended_by".to_string(),
                            Value::String(comment_core.suspended_by.clone().unwrap())
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

async fn get_comment_stat(
    user_id: &str,
    db: &Database,
    sub_fields: Vec<&str>
) -> Result<serde_json::Value, HttpResponse> {
    let collection = db.collection::
    <Comment::CommentStat>("comment_stat");
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
        let comment_stat = option.unwrap();
        let mut response = Map::new();
        for field in sub_fields {
            match field {
                "like_count" => response.insert(
                    "like_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        comment_stat.like_count.clone()
                    ))
                ),
                "reply_count" => response.insert(
                    "reply_count".to_string(),
                    Value::Number(serde_json::Number::from(
                        comment_stat.reply_count.clone()
                    ))
                ),
                "modified_at" => response.insert(
                    "modified_at".to_string(),
                    Value::Number(serde_json::Number::from(
                        comment_stat.modified_at.clone()
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