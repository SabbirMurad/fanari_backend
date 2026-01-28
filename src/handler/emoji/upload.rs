use uuid::Uuid;
use chrono::Utc;
use mongodb::bson::doc;
use crate::builtins::sqlite;
use crate::BuiltIns::mongo::MongoDB;
use crate::model::Emoji;
// use crate::Integrations::Firebase;
use serde::{ Serialize, Deserialize };
use crate::utils::response::Response;
use actix_web::{web, Error, HttpResponse, HttpRequest};
use crate::middleware::auth::{require_access, AccessRequirement};
use crate::model::Account::AccountRole;
use image::io::Reader as ImageReader;
use webp::Encoder;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReqBody {
    name: String,
    serial: u64,
    data: Vec<u8>,
}

pub async fn task(
    req: HttpRequest,
    form_data: web::Json<Vec<ReqBody>>
) -> Result<HttpResponse, Error> {
    let user = require_access(
        &req,
        AccessRequirement::Role(AccountRole::Administrator)
    )?;

    let user_id = user.user_id;

    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }

    let sqlite_conn = sqlite::connect(sqlite::DBF::IMG).unwrap();

    let now = Utc::now().timestamp_millis();

    for emoji_item in form_data.iter() {
        let uuid = Uuid::new_v4().to_string();

        let emoji_data = Emoji {
            uuid: uuid.clone(),
            name: emoji_item.name.clone(),
            serial: emoji_item.serial.clone() as usize,
            created_at: now,
            created_by: user_id.clone(),
            modified_at: now,
            modified_by: user_id.clone(),
        };

        let collection = db.collection::<Emoji>("emoji");
        let result = collection.insert_one(emoji_data).await;
        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        // Converting to webp
        let webp_bytes = match convert_to_webp(emoji_item.data.clone()) {
            Ok(bytes) => bytes,
            Err(error) => {
                log::error!("{:?}", error);
                session.abort_transaction().await.unwrap();
                return Ok(Response::internal_server_error(&error.to_string()));
            },
        };

            // Uploading image to sqlite
        let result = sqlite_conn.execute("
            INSERT INTO emoji (uuid, original, webp)
            VALUES (?1, ?2, ?3)",
            (
                &uuid,
                &emoji_item.data,
                &webp_bytes
            )
        );

        if let Err(error) = result {
            log::error!("{:?}", error);
            session.abort_transaction().await.unwrap();
            return Ok(Response::internal_server_error(&error.to_string()));
        }
    }

    /* DATABASE ACID COMMIT */
    if let Err(error) = session.commit_transaction().await {
        log::error!("{:?}", error);
        return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    Ok(
        HttpResponse::Ok()
        .content_type("application/json")
        .json(Response { message: "Successfully Uploaded".to_string() })
    )
}

fn convert_to_webp(image_bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    // Decode the image (autodetects format)
    let reader = match ImageReader::new(std::io::Cursor::new(image_bytes))
    .with_guessed_format() {
        Ok(reader) => reader,
        Err(error) => return Err(error.to_string()),
    };

    let img = match reader.decode() {
        Ok(img) => img,
        Err(error) => return Err(error.to_string()),
    };

    // Convert to RGBA (preserves transparency)
    let rgba = img.to_rgba8();

    // Encode to WebP with quality 80
    let encoder = Encoder::from_rgba(&rgba, img.width(), img.height());
    let webp = encoder.encode(80.0);

    Ok(webp.to_vec())
}