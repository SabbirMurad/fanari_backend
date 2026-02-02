use actix_web::web;
use crate::Handler;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/conversation")
        //Create Single Conversation
        .route(
          "/single",
          web::post().to(Handler::Conversation::CreateSingle::task)
        )
        //Create Group Conversation
        .route(
          "/group",
          web::post().to(Handler::Conversation::CreateGroup::task)
        )
        //List Conversation
        .route(
          "/list",
          web::get().to(Handler::Conversation::List::task)
        )
    );
}