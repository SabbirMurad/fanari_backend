use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/reply")
        //Create
        .route(
          "",
          web::post().to(Handler::Reply::Create::task)
        )
        //Get
        .route(
          "",
          web::get().to(Handler::Reply::Get::task)
        )
        //Delete
        .route(
          "/{uuid}",
          web::delete().to(Handler::Reply::Delete::task)
        )
    );
}