use actix::Addr;
use mongodb::bson::doc;
use futures::StreamExt;
use actix_web_actors::ws;
use super::WsHandler::WsConn;
use serde::{Deserialize, Serialize};
use crate::{model::Conversation, utils::response::Response};
use crate::builtins::mongo::MongoDB;
use crate::Handler::WebSocket::lobby::Lobby;
use actix_web::{Error, HttpRequest, HttpResponse, web::{Data, Payload}};
use crate::Middleware::Auth::{require_access, AccessRequirement};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConversationRetrieve { conversation_id: String }


pub async fn task(
  req: HttpRequest,
  stream: Payload,
  srv: Data<Addr<Lobby>>
) -> Result<HttpResponse, Error> {
  let user = require_access(
      &req,
      AccessRequirement::AnyToken
  )?;

  let user_id = user.user_id;

  println!("{} connected\n", user_id);

  let mut group_ids = Vec::new();
  let db = MongoDB.connect();

  let collection = db.collection::<ConversationRetrieve>("conversation_participant");

  let result = collection.find(
    doc!{"user_id": &user_id},
  ).await;
  
  if let Err(error) = result {
    log::error!("{:?}", error);
    return Ok(Response::internal_server_error(&error.to_string()));
  }
  
  let mut cursor = result.unwrap();
  while let Some(result) = cursor.next().await {
    if let Err(error) = result {
      log::error!("{:?}", error);
      return Ok(Response::internal_server_error(&error.to_string()));
    }
  
    let conversation = result.unwrap();
    group_ids.push(conversation.conversation_id);
  }

  let ws = WsConn::new(
    &user_id,
    group_ids,
    srv.get_ref().clone()
  );

  match ws::start(ws, &req, stream) {
    Ok(response) => Ok(response),
    Err(error) => {
      log::error!("{:?}", error);
      Ok(HttpResponse::InternalServerError().body(error.to_string()))
    },
  }
}