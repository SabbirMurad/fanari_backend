use std::collections::{HashMap, HashSet};
use actix::prelude::{Actor, Context, Handler, Recipient};
use mongodb::bson::doc;
use crate::{Model, builtins::mongo::MongoDB, handler::web_socket::message::WsEnvelope};

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
    fn send_message(&self, message: &WsEnvelope, send_to: &str) {
        if let Some(socket) = self.sessions.get(send_to) {
            let message = serde_json::to_string(&message).unwrap();

            let _ = socket.do_send(WsMessage(message.to_string()));
        }
        else {
            log::error!(
                "Attempted to send message to non-existent user: {}", send_to
            );
        }
    }

    fn broadcast_to_room(
        &self,
        message: &WsEnvelope,
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

    fn broadcast_to_call(
        &self,
        message: &WsEnvelope,
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

    // Builds a JSON envelope string — single place for all outgoing messages
    fn make_envelope(msg_type: &str, payload: serde_json::Value) -> String {
        serde_json::json!({
            "type": msg_type,
            "payload": payload,
        }).to_string()
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}


impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, disconnect: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        if self.sessions.remove(&disconnect.user_id).is_some() {
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

            let message = WsEnvelope {
                msg_type: "disconnect".to_string(),
                payload: serde_json::json!({
                    "user_id": disconnect.user_id,
                })
            };

            for user_id in autndm {
                self.send_message(&message, &user_id);
            }

            actix::spawn(async move {
                let collection = MongoDB.connect()
                    .collection::<Model::Account::AccountStatus>("account_status");
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
        let mut autncm = Vec::new();

        for room_id in &connect.rooms {
            self.rooms
                .entry(room_id.clone())
                .or_insert_with(HashSet::new)
                .insert(connect.user_id.clone());

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

        let message = WsEnvelope {
            msg_type: "connect".to_string(),
            payload: serde_json::json!({
                "user_id": connect.user_id,
            })
        };

        for user_id in autncm {
            self.send_message(&message, &user_id);
        }

        self.sessions.insert(connect.user_id.clone(), connect.addr);

        actix::spawn(async move {
            let collection = MongoDB.connect()
                .collection::<Model::Account::AccountStatus>("account_status");
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
                self.send_message(
                    &msg.msg,
                    conn_id
                )
            });
    }
}


impl Handler<DirectMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: DirectMessage, _ctx: &mut Self::Context) -> Self::Result {
        if !self.sessions.contains_key(&msg.to_user_id) {
            let message = WsEnvelope {
                msg_type: "call_signal".to_string(),
                payload: serde_json::json!({
                    "type": "peer_offline",
                    "from": msg.to_user_id,
                })
            };

            self.send_message(&message, &msg.from_user_id);
            return;
        }

        self.send_message(&msg.msg, &msg.to_user_id);
    }
}


impl Handler<RoomSignalMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: RoomSignalMessage, _ctx: &mut Self::Context) -> Self::Result {
        let envelope = msg.msg.clone();

        // Signal type lives inside payload now, not at the top level
        let signal_type = envelope.payload["type"].as_str().unwrap_or("");

        match signal_type {
            "call_start" => {
                self.active_calls
                    .entry(msg.room_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(msg.from_user_id.clone());

                self.broadcast_to_room(
                    &msg.msg,
                    &msg.room_id,
                    &msg.from_user_id
                );
            }

            "call_join" => {
                let existing_participants: Vec<String> = self.active_calls
                    .get(&msg.room_id)
                    .map(|set| set.iter().cloned().collect())
                    .unwrap_or_default();

                self.active_calls
                    .entry(msg.room_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(msg.from_user_id.clone());

                let participant_msg = WsEnvelope {
                    msg_type: "call_signal".to_string(),
                    payload: serde_json::json!({
                        "type": "call_participants",
                        "from": "system",
                        "participants": existing_participants,
                        "room_id": msg.room_id,
                    })
                };

                self.send_message(&participant_msg, &msg.from_user_id);

                self.broadcast_to_call(
                    &msg.msg,
                    &msg.room_id,
                    &msg.from_user_id
                );
            }

            "call_leave" => {
                if let Some(call) = self.active_calls.get_mut(&msg.room_id) {
                    call.remove(&msg.from_user_id);
                    if call.is_empty() {
                        self.active_calls.remove(&msg.room_id);
                    }
                }

                self.broadcast_to_call(
                    &msg.msg,
                    &msg.room_id,
                    &msg.from_user_id
                );
            }

            "video_toggle" | "audio_toggle" => {
                self.broadcast_to_call(
                    &msg.msg,
                    &msg.room_id,
                    &msg.from_user_id
                );
            }

            _ => {
                log::warn!("Unknown RoomSignalMessage type: {}", signal_type);
            }
        }
    }
}