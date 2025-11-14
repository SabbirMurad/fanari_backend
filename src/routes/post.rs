use actix_web::web;
use crate::Handler;
use crate::middleware::auth::AccessRequirement;

pub fn router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/post")
        //Create
        .service(
            web::resource("")
                .app_data(web::Data::new(AccessRequirement::AnyToken))
                //Create Post
                .route(web::post().to(Handler::Post::Create::task))
                //Get Details
                .route(web::get().to(Handler::Post::Get::task))
        )
        // .service(
        //     web::resource("")
        //         .app_data(web::Data::new(AccessRequirement::AnyToken))
        // )
        //Delete
        .route(
          "/{uuid}",
          web::delete().to(Handler::Post::Delete::task)
        )
    );
}