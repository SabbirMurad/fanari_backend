use mongodb::bson::doc;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{ web, Error, HttpRequest, HttpResponse};
use crate::builtins::image::{add, ImageFrom};
use crate::Middleware::Auth::{require_access, AccessRequirement};

use crate::Model::VideoStruct;


#[derive(Debug, Deserialize, Serialize, Clone)]
struct PostData {
  video_id: String,
  image: Vec<u8>,
}

pub async fn task(req: HttpRequest, form_data: web::Json<PostData>) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::AnyToken
    )?;

    let user_id = user.user_id;

    let result = add(
        Some(form_data.video_id.clone()),
        form_data.image.clone(),
        ImageFrom::VideoThumbnail
    ).await;
    if let Err(err) = result {
      log::error!("{:?}", err);
      return Ok(Response::internal_server_error(&err));
    }

    let image_info = result.unwrap();
    let video_thumbnail = VideoStruct {
      uuid: image_info.uuid,
      height: image_info.height,
      width: image_info.width,
      thumbnail_type: image_info.r#type,
    };

  Ok(HttpResponse::Ok().content_type("application/json").json(video_thumbnail))
}