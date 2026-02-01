use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/api/ws")
    .route(
      "/chat",
      web::get().to(Handler::WebSocket::Connect::task)
    )
  );
}