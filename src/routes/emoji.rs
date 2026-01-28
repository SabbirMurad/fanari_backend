use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/emoji")
        //Create
        .route(
            "",
            web::post().to(Handler::Emoji::Upload::task)
        )
        .route(
            "/list",
            web::get().to(Handler::Emoji::List::task)
        )
        .route(
            "/webp/{image_id}",
            web::get().to(Handler::Emoji::Webp::task)
        )
        .route(
            "/original/{image_id}",
            web::get().to(Handler::Emoji::Original::task)
        )
        .route(
            "/metadata/{image_id}", 
            web::get().to(Handler::Emoji::Metadata::task)
        )
    );
}