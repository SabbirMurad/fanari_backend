use std::collections::{HashMap, HashSet};
use actix::prelude::{Actor, Context, Handler, Recipient};
use mongodb::bson::doc;
use crate::{builtins::mongo::MongoDB, Model};

use super::message::{
    Connect,
    Disconnect,
    WsMessage,
    ClientActorMessage,
    DirectMessage,
    RoomSignalMessage
};


pub type Socket = Recipient<WsMessage>;

#[derive(Clone)]
pub struct Lobby {
    pub sessions: HashMap<String, Socket>,
    pub rooms: HashMap<String, HashSet<String>>,

    // ✅ NEW — tracks active group calls per room
    // room_id → set of user_ids currently in the call
    pub active_calls: HashMap<String, HashSet<String>>,
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            active_calls: HashMap::new(),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: &str, send_to: &str) {
        if let Some(socket) = self.sessions.get(send_to) {
            let _ = socket.do_send(WsMessage(message.to_string()));
        }
        else {
            log::error!(
                "Attempted to send message to non-existent user: {}", send_to
            );
        }
    }

    // ✅ NEW — sends to everyone in a room except the sender
    fn broadcast_to_room(
        &self,
        message: &str,
        room_id: &str,
        except_user_id: &str
    ) {
        if let Some(members) = self.rooms.get(room_id) {
            for user_id in members {
                if user_id != except_user_id {
                    self.send_message(message, user_id);
                }
            }
        }
    }

    // ✅ NEW — sends to everyone currently IN THE CALL except the sender
    fn broadcast_to_call(
        &self,
        message: &str,
        room_id: &str,
        except_user_id: &str
    ) {
        if let Some(participants) = self.active_calls.get(room_id) {
            for user_id in participants {
                if user_id != except_user_id {
                    self.send_message(message, user_id);
                }
            }
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

// ✅ NEW — Direct user-to-user (for Offer/Answer/IceCandidate in both 1-1 and group)
impl Handler<DirectMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: DirectMessage, _ctx: &mut Self::Context) -> Self::Result {
        if !self.sessions.contains_key(&msg.to_user_id) {
            self.send_message(
                &format!(
                    "%call_signal%::{{\"type\":\"PeerOffline\",\"from\":\"{}\"}}",
                    msg.to_user_id
                ),
                &msg.from_user_id,
            );
            return;
        }
      
        self.send_message(&msg.msg, &msg.to_user_id);
    }
}

// ✅ NEW — Room-wide broadcast (for CallStart/CallJoin/CallLeave/VideoToggle)
impl Handler<RoomSignalMessage> for Lobby {
  type Result = ();

  fn handle(&mut self, msg: RoomSignalMessage, _ctx: &mut Self::Context) -> Self::Result {
    // Parse just enough to know the signal type
    let parsed: serde_json::Value = match serde_json::from_str(&msg.msg
      .strip_prefix("%call_signal%::")
      .unwrap_or(&msg.msg)
    ) {
      Ok(v) => v,
      Err(e) => {
        log::error!("Failed to parse RoomSignalMessage: {:?}", e);
        return;
      }
    };

    let signal_type = parsed["type"].as_str().unwrap_or("");

    match signal_type {

      // Someone is starting a brand new call in this room
      "CallStart" => {
        let call = self.active_calls
          .entry(msg.room_id.clone())
          .or_insert_with(HashSet::new);
        call.insert(msg.from_user_id.clone());

        // Notify everyone in the room (not just call participants yet)
        self.broadcast_to_room(&msg.msg, &msg.room_id, &msg.from_user_id);
      }

      // Someone is joining an ongoing call
      "CallJoin" => {
        // Get current participants BEFORE adding the new joiner
        // so we can tell the new joiner who's already there
        let existing_participants: Vec<String> = self.active_calls
          .get(&msg.room_id)
          .map(|set| set.iter().cloned().collect())
          .unwrap_or_default();

        // Add joiner to the active call
        self.active_calls
          .entry(msg.room_id.clone())
          .or_insert_with(HashSet::new)
          .insert(msg.from_user_id.clone());

        // Tell the new joiner who they need to create offers to
        let participant_list = serde_json::json!({
          "type": "CallParticipants",
          "from": "system",
          "participants": existing_participants,
          "room_id": msg.room_id,
        });
        let participant_msg = format!("%call_signal%::{}", participant_list);
        self.send_message(&participant_msg, &msg.from_user_id);

        // Tell everyone already in the call that a new person joined
        self.broadcast_to_call(&msg.msg, &msg.room_id, &msg.from_user_id);
      }

      // Someone is leaving the call (but staying in the chat room)
      "CallLeave" => {
        if let Some(call) = self.active_calls.get_mut(&msg.room_id) {
          call.remove(&msg.from_user_id);

          // If the call is now empty, clean it up
          if call.is_empty() {
            self.active_calls.remove(&msg.room_id);
          }
        }

        // Tell remaining participants to close their connection to this peer
        self.broadcast_to_call(&msg.msg, &msg.room_id, &msg.from_user_id);
      }

      // Camera/mic toggle — only relevant to current call participants
      "VideoToggle" | "AudioToggle" => {
        self.broadcast_to_call(&msg.msg, &msg.room_id, &msg.from_user_id);
      }

      _ => {
        log::warn!("Unknown RoomSignalMessage type: {}", signal_type);
      }
    }
  }
}