use mongodb::bson::doc;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Middleware::Auth::{require_access, AccessRequirement};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqBody {
  field_1: String,
  field_2: String,
}
// TODO:

pub async fn task(req: HttpRequest, req_body: web::Json<ReqBody>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

  Ok(HttpResponse::Ok().content_type("application/json").json(Response{
    message: "Successfully Created".to_string(),
  }))
}