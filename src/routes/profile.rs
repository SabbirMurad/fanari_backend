use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/profile")
        //Get
        .route(
          "/list",
          web::post().to(Handler::Profile::List::task)
        )
        .route(
          "/myself/details",
          web::get().to(Handler::Profile::Myself::task)
        )
        //Update
        .route(
          "",
          web::patch().to(Handler::Profile::Update::task)
        )
    );
}