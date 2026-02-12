use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/support")
        .route(
            "/start",
            web::get().to(Handler::Support::Start::task)
        )
        .route(
            "/end",
            web::post().to(Handler::Support::End::task)
        )
        .route(
            "/user-text",
            web::post().to(Handler::Support::UserText::task)
        )
        .route(
            "/support-text",
            web::post().to(Handler::Support::SupportText::task)
        )
    );
}