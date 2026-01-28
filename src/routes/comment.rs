use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/comment")
        //Create
        .route(
          "",
          web::post().to(Handler::Comment::Create::task)
        )
        //Get
        .route(
          "",
          web::get().to(Handler::Comment::Get::task)
        )
        //Delete
        .route(
          "/{uuid}",
          web::delete().to(Handler::Comment::Delete::task)
        )
    );
}