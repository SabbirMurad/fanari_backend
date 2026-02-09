use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/image")
        .app_data(web::JsonConfig::default().limit(64 * 8 * 1024 * 1024))//50 MB
        .route(
            "",
            web::post().to(Handler::Image::Upload::task)
        )
        .route(
            "/webp/{image_id}",
            web::get().to(Handler::Image::Webp::task)
        )
        .route(
            "/original/{image_id}",
            web::get().to(Handler::Image::Original::task)
        )
        .route(
            "/metadata/{image_id}", 
            web::get().to(Handler::Image::Metadata::task)
        )
        .route(
            "/metadata", 
            web::post().to(Handler::Image::metadata_bulk::task)
        )
    );
}