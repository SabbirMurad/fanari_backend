use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/report")
        .route(
            "/create",
            web::post().to(Handler::Report::Create::task)
        )
        .route(
            "/resolve",
            web::post().to(Handler::Report::Resolve::task)
        )
    );
}