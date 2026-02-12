use uuid::Uuid;
use chrono::Utc;
use mongodb::bson::doc;
use crate::builtins::mongo::MongoDB;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::Model::report::{ReportStatus, Report, ReportType, ReportedOn};
use crate::Middleware::Auth::{require_access, AccessRequirement};


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostData {
  pub r#type: ReportType,
  pub reported_on: ReportedOn,
  pub reported_uuid: String,
  pub reason: String,
}

pub async fn task(req: HttpRequest, form_data: web::Json<PostData>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let collection = MongoDB.connect().collection::<Report>("report");
    let report = Report {
        uuid: Uuid::new_v4().to_string(),
        owner: user_id,
        reason: form_data.reason.clone(),
        reported_on: form_data.reported_on.clone(),
        reported_uuid: form_data.reported_uuid.clone(),
        reply: None,
        resolved_at: None,
        resolved_by: None,
        status: ReportStatus::Pending,
        r#type: form_data.r#type.clone(),
        created_at: Utc::now().timestamp_millis(),
    };

    let result = collection.insert_one(
        report,
    ).await;

    if let Err(error) = result {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    Ok(HttpResponse::Ok().content_type("application/json").json(Response{
        message: "Successfully Created".to_string(),
    }))
}