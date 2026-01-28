use crate::utils::response::Response;
use crate::builtins::sqlite;
use actix_web::{web, Error, HttpResponse};

pub async fn task(image_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let image_id = image_id.trim().to_string();
    let sqlite_conn = sqlite::connect(sqlite::DBF::IMG).unwrap();

    let result = sqlite_conn.query_row(
        "SELECT original FROM emoji WHERE uuid = ?1",
        [&image_id],
        |row| row.get::<usize, Vec<u8>>(0)
    );

    let image_data = match result {
        Ok(data) => data,
        Err(error) => return Ok(
            Response::internal_server_error(&error.to_string())
        )
    };

    Ok(
        HttpResponse::Ok()
        .content_type("image/png")
        .body(image_data)
    )
}