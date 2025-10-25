use actix_web::{web, Error, HttpResponse};

pub async fn task(image_id: web::Path<String>) -> Result<HttpResponse, Error> {
    println!("{image_id}");

    Ok(HttpResponse::Ok().content_type("image/webp").json(serde_json::json!({
        "full_name": "Sabbir Hassan",
    })))
}