use actix::prelude::{Message, Recipient};

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
  pub addr: Recipient<WsMessage>,
  pub rooms: Vec<String>,
  pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
  pub rooms: Vec<String>,
  pub user_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientActorMessage {
  pub user_id: String,
  pub msg: String,
  pub room_id: String,
}