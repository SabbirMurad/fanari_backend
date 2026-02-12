use mongodb::bson::doc;
use crate::builtins::sqlite::{self, Sqlite3};
use serde::{ Serialize, Deserialize };
use actix_web::{ Error, web, HttpResponse};


#[derive(Debug, Serialize, Deserialize)]
struct FetchPayload { data: Vec<u8>, r#type: String }

pub async fn task(video_id: web::Path<String>) -> Result<HttpResponse, Error> {
    let db_conn = Sqlite3::connect(sqlite::DBF::IMG).unwrap();
    let mut stmt = db_conn.prepare_cached(
        "SELECT data, type FROM thumbnail WHERE uuid = ?1"
    ).unwrap();

    match stmt.query_row(
        [video_id],
        |row| {
            Ok(FetchPayload {
                data: row.get(0)?,
                r#type: row.get(1)?,
            })
        }
    ) {
        Ok(result) => {
            Ok(HttpResponse::Ok().content_type(result.r#type).body(result.data))
        }
        Err(error) => {
            log::error!("{:?}", error);
            Ok(HttpResponse::NotFound().finish())
        }
    }
}