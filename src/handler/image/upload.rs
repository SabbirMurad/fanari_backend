use uuid::Uuid;
use chrono::Utc;
use futures_util::StreamExt as _;
use std::collections::HashMap;
use crate::model::ImageStruct;
use actix_multipart::Multipart;
use crate::builtins::mongo::MongoDB;
use crate::utils::response::Response;
use actix_web::{Error, HttpResponse};
use image::io::Reader as ImageReader;
// use image::DynamicImage::;
use webp::Encoder;


pub async fn task(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let mut images_data = Vec::new();
    let mut text_fields = HashMap::new();

    // Iterate over multipart fields
    while let Some(item) = payload.next().await {
        let mut field = item?;

        // You can check the field name if you have multiple fields
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => {
                return Ok(Response::bad_request("Missing content disposition"))
            },
        };

        let field_name = match content_disposition.get_name() {
            Some(name) => name,
            None => {
                return Ok(Response::bad_request("Missing field name"))
            },
        };

        match content_disposition.get_filename() {
            Some(name) => {
                // Collect the bytes of the image
                let mut image_bytes: Vec<u8> = Vec::new();

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    image_bytes.extend_from_slice(&data);
                }

                images_data.push((name.clone(), name.to_string(), image_bytes));
            },
            None => {
                // It's a text field
                let mut value = Vec::new();
                while let Some(chunk) = field.next().await {
                    value.extend_from_slice(&chunk?);
                }

                text_fields.insert(
                    field_name.clone(),
                    String::from_utf8_lossy(&value).to_string()
                );
            },
        };
    }

    let mut image_ids: Vec<String> = Vec::new();
    let db = MongoDB.connect();
    let collection = db.collection::<ImageStruct>("image");
    let created_at = Utc::now().timestamp_millis();

    for (i, (_field_name, filename, bytes)) in images_data.iter().enumerate() {
        let blur_hash_key = format!("blur_hash_{}", i);
        let width_key = format!("width_{}", i);
        let height_key = format!("height_{}", i);

        let blur_hash = text_fields
            .get(blur_hash_key.as_str())
            .unwrap();
 
        let width = text_fields
            .get(width_key.as_str())
            .unwrap();

        let height = text_fields
            .get(width_key.as_str())
            .unwrap();

        // Converting to webp
        let webp_bytes = match convert_to_webp(bytes.clone()) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Ok(Response::internal_server_error(&error.to_string()));
            },
        };

        // Creating the metadata in mongo
        let uuid = Uuid::now_v7().to_string();

        let image_doc = ImageStruct {
            uuid: uuid.clone(),
            blur_hash: blur_hash.clone(),
            width: width.parse().unwrap(),
            height: height.parse().unwrap(),
            created_at,
            original_size: bytes.len(),
            webp_size: webp_bytes.len(),
            used_at: None,
            temporary: true,
            original_type: "image".to_string(),
        };

        let result = collection.insert_one(image_doc.clone()).await;
        if let Err(error) = result {
            return Ok(Response::internal_server_error(&error.to_string()));
        }

        // Storing the id for the response
        image_ids.push(uuid.clone());
    }

    Ok(HttpResponse::Ok().content_type("application/json").json(image_ids))
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