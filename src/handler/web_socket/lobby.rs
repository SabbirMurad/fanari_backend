use std::collections::{HashMap, HashSet};
use actix::prelude::{Actor, Context, Handler, Recipient};
use mongodb::bson::doc;
use crate::{builtins::mongo::MongoDB, Model};

use super::message::{Connect, Disconnect, WsMessage, ClientActorMessage};


pub type Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct Lobby {
  pub sessions: HashMap<String, Socket>,
  pub rooms: HashMap<String, HashSet<String>>,
}

impl Default for Lobby {
  fn default() -> Self {
    Self {
      sessions: HashMap::new(),
      rooms: HashMap::new(),
    }
  }
}

impl Lobby {
  fn send_message(&self, message: &str, send_to: &str) {
    if let Some(socket) = self.sessions.get(send_to) {
      let _ = socket.do_send(WsMessage(message.to_string()));
    }
    else {
      log::error!("Attempted to send message to non-existent user: {}", send_to);
    }
  }
}

impl Actor for Lobby {
  type Context = Context<Self>;
}


impl Handler<Disconnect> for Lobby {
  type Result = ();

  fn handle(&mut self, disconnect: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
    if self.sessions.remove(&disconnect.user_id).is_some() {
      // autndm - all_user_that_need_disconnect_message
      let mut autndm = Vec::new();

      for room_id in &disconnect.rooms {
        self.rooms.get(room_id)
          .unwrap()
          .iter()
          .filter(|conn_id| *conn_id.to_owned() != disconnect.user_id)
          .for_each(|user_id| {
            let user = user_id.to_owned();
            if !autndm.contains(&user) {
              autndm.push(user);
            }
          });
  
        if let Some(lobby) = self.rooms.get_mut(room_id) {
          if lobby.len() > 1 {
            lobby.remove(&disconnect.user_id);
          }
          else {
            self.rooms.remove(room_id);
          }
        }
      }

      for user_id in autndm {
        self.send_message(&format!("%disconnect%::{}", disconnect.user_id), &user_id);
      }

      actix::spawn(async move {
        let collection = MongoDB.connect().collection::<Model::Account::AccountStatus>("account_status");
        let result = collection.update_one(
          doc!{"uuid": &disconnect.user_id},
          doc!{"$set":{
            "online": false,
            "last_seen": chrono::Utc::now().timestamp_millis(),
          }},
        ).await;
        
        if let Err(error) = result {
          log::error!("{:?}", error);
        }
      });
    }
  }
}

impl Handler<Connect> for Lobby {
  type Result = ();

  fn handle(&mut self, connect: Connect, _ctx: &mut Self::Context) -> Self::Result {
    // autncm - all_user_that_need_connect_message
    let mut autncm = Vec::new();

    for room_id in &connect.rooms {
      self.rooms.entry(room_id.clone()).or_insert_with(HashSet::new).insert(connect.user_id.clone());
  
      self.rooms.get(&room_id.clone())
        .unwrap()
        .iter()
        .filter(|conn_id| *conn_id.to_owned() != connect.user_id)
        .for_each(|conn_id| {
          let user = conn_id.to_owned();
          if !autncm.contains(&user) {
            autncm.push(user);
          }
        });
    }

    for user_id in autncm {
      self.send_message(&format!("%connect%::{}", connect.user_id), &user_id);
    }

    self.sessions.insert(connect.user_id.clone(), connect.addr);

    actix::spawn(async move {
      let collection = MongoDB.connect().collection::<Model::Account::AccountStatus>("account_status");
      let result = collection.update_one(
        doc!{"uuid": &connect.user_id},
        doc!{"$set":{
          "online": true
        }},
      ).await;
      
      if let Err(error) = result {
        log::error!("{:?}", error);
      }
    });
  
    // self.send_message(&format!("Your id is {}", connect.user_id), &connect.user_id);
  }
}


impl Handler<ClientActorMessage> for Lobby {
  type Result = ();

  fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Self::Context) -> Self::Result {
    self.rooms
      .get(&msg.room_id)
      .unwrap()
      .iter()
      .for_each(|conn_id| {
        self.send_message(&msg.msg, conn_id)
      });
  }
}