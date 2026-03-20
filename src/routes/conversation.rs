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
        //List Texts in a Conversation
        .route(
          "/text/list",
          web::get().to(Handler::Conversation::TextList::task)
        )
        //Favorite Conversation
        .route(
          "/favorite",
          web::patch().to(Handler::Conversation::Favorite::task)
        )
        //Mute Conversation
        .route(
          "/mute",
          web::patch().to(Handler::Conversation::Mute::task)
        )
        //Delete Conversation
        .route(
          "/delete/{uuid}",
          web::delete().to(Handler::Conversation::Delete::task)
        )
        //Block Conversation
        .route(
          "/block",
          web::post().to(Handler::Conversation::Block::task)
        )
    );
}