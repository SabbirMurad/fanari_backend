use actix_web::{Error, HttpResponse};

pub async fn task() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}