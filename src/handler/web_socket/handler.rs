use rand::rand_core::le;
use uuid::Uuid;
use chrono::Utc;
use super::Lobby::Lobby;
use mongodb::bson::doc;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use crate::{builtins::mongo::MongoDB, model::conversation::MessageRead};
use serde::{Deserialize, Serialize};
use crate::handler::web_socket::message::{WsEnvelope, WsEnvelopeType};

use super::WsMessage::{
    ClientActorMessage,
    Connect,
    Disconnect,
    WsMessage,
    DirectMessage,
    RoomSignalMessage
};
use serde_json::Value;

use actix::{
    fut,
    Actor,
    ActorContext,
    ActorFutureExt,
    Addr,
    AsyncContext,
    ContextFutureSpawner,
    Handler,
    Running,
    StreamHandler,
    WrapFuture,
};

use crate::Model::{
    Conversation,
    AudioStruct,
    AttachmentStruct,
    Mention
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SocketIncomingTextModel {
    conversation_id: String,
    text: Option<String>,
    images: Option<Vec<String>>,
    audio: Option<AudioStruct>,
    video: Option<String>,
    attachment: Option<AttachmentStruct>,
    r#type: Conversation::TextType,
    reply_to: Option<String>,
    mentions: Option<Vec<Mention>>,
    emoji: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SocketOutgoingTextModel {
    uuid: String,
    owner: String,
    conversation_id: String,
    text: Option<String>,
    mentions: Option<Vec<Mention>>,
    images: Option<Vec<String>>,
    audio: Option<AudioStruct>,
    video: Option<String>,
    attachment: Option<AttachmentStruct>,
    emoji: Option<String>,
    r#type: Conversation::TextType,
    reply_to: Option<String>,
    seen_by: Vec<String>,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
struct IncomingSignal {
    r#type: String,
    to: Option<String>,       // present for directed signals
    room_id: Option<String>,  // present for room-wide signals
    sdp: Option<String>,
    candidate: Option<serde_json::Value>,
    call_type: Option<String>,
    enabled: Option<bool>,
    muted: Option<bool>,
}

#[derive(Deserialize)]
struct TypingPayload {
    conversation_id: String,
    user_id: String,
    name: String,
}

#[derive(Deserialize, Clone)]
struct MessageSeenPayload {
    conversation_id: String,
    text_ids: Vec<String>,
}

pub struct WsConn {
    rooms: Vec<String>,
    lobby_addr: Addr<Lobby>,
    hb: Instant,
    user_id: String,
}

impl WsConn {
    pub fn new(user_id: &str, rooms: Vec<String>, lobby_addr: Addr<Lobby>) -> Self {
        Self {
            user_id: user_id.to_string(),
            rooms,
            lobby_addr,
            hb: Instant::now(),
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        
        let addr = ctx.address();
        self.lobby_addr.send(Connect {
            addr: addr.recipient(),
            rooms: self.rooms.clone(),
            user_id: self.user_id.clone()
        })
        .into_actor(self)
        .then(|res, _act, ctx| {
            match res {
                Ok(_res) => (),
                _ => ctx.stop(),
            }
            fut::ready(())
        })
        .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        self.lobby_addr.do_send(Disconnect {
            user_id: self.user_id.clone(),
            rooms: self.rooms.clone()
        });

        Running::Stop
    }
}

impl WsConn {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting due to failed heartbeat: {:?}", act.user_id);
                act.lobby_addr.do_send(Disconnect {
                user_id: act.user_id.clone(),
                rooms: act.rooms.clone()
                });

                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }

   fn handle_text_message(&mut self, raw_str: String) {
        // Single parse point — no splitting, no prefix matching
        let envelope: Result<WsEnvelope, _> = serde_json::from_str(&raw_str);

        if let Err(error) = envelope {
            log::error!("Invalid envelope: {:?}", error);
            return;
        }

        let envelope = envelope.unwrap();

        match envelope.msg_type {
            WsEnvelopeType::text => self.handle_text(envelope.payload),
            WsEnvelopeType::typing => self.handle_typing(envelope.payload),
            WsEnvelopeType::message_seen => self.handle_message_seen(
                envelope.payload
            ),
            WsEnvelopeType::call_signal => self.handle_call_signal(
                envelope.payload
            ),
            _   => log::warn!("Unknown message prefix: {}", envelope.msg_type),
        }
    }


    fn handle_text(&mut self, payload: Value) {
        let incoming_text: Result<SocketIncomingTextModel, _> = serde_json::from_value(payload);

        if let Err(error) = incoming_text {
            log::error!("Invalid payload structure: {:?}", error);
            return;
        }

        let incoming_text = incoming_text.unwrap();

        let room_id = incoming_text.conversation_id.clone();

        let outgoing_message = SocketOutgoingTextModel {
            uuid:            Uuid::new_v4().to_string(),
            owner:           self.user_id.clone(),
            conversation_id: room_id.clone(),
            text:            incoming_text.text.clone(),
            emoji:           incoming_text.emoji.clone(),
            mentions:        incoming_text.mentions.clone(),
            images:          incoming_text.images.clone(),
            audio:           incoming_text.audio.clone(),
            video:           incoming_text.video.clone(),
            reply_to:        incoming_text.reply_to.clone(),
            seen_by:         vec![],
            created_at:      Utc::now().timestamp_millis(),
            r#type:          incoming_text.r#type,
            attachment:      incoming_text.attachment.clone(),
        };

        let outgoing_message_clone = outgoing_message.clone();
        actix::spawn(async move {
            save_message_in_database(outgoing_message_clone).await;
        });

        

        self.lobby_addr.do_send(ClientActorMessage {
            user_id: self.user_id.clone(),
            room_id,
            msg: WsEnvelope {
                msg_type: WsEnvelopeType::text,
                payload:  serde_json::to_value(outgoing_message).unwrap(),
            },
        });
    } 

    fn handle_typing(&mut self, payload: Value) {
        let typing: Result<TypingPayload, _> =
        serde_json::from_value(payload);

        if let Err(error) = typing {
            log::error!("Invalid payload structure: {:?}", error);
            return;
        }

        let typing = typing.unwrap();

        self.lobby_addr.do_send(ClientActorMessage {
            user_id: typing.user_id.clone(),
            room_id: typing.conversation_id.clone(),
            msg: WsEnvelope {
                msg_type: WsEnvelopeType::typing,
                payload:  serde_json::json!({
                    "conversation_id": typing.conversation_id,
                    "user_id": typing.user_id,
                    "name": typing.name
                }),
            },
        });
    }

    fn handle_message_seen(&mut self, payload: Value) {
        let message_seen: Result<MessageSeenPayload, _> =
        serde_json::from_value(payload);

        if let Err(error) = message_seen {
            log::error!("Invalid payload structure: {:?}", error);
            return;
        }

        let message_seen = message_seen.unwrap();

        let user_id = self.user_id.clone();
        let message_seen_clone = message_seen.clone();
        actix::spawn(async move {
            add_message_seen(message_seen_clone, &user_id).await;
        });

        self.lobby_addr.do_send(ClientActorMessage {
            user_id: self.user_id.clone(),
            room_id: message_seen.conversation_id.clone(),
            msg: WsEnvelope {
                msg_type: WsEnvelopeType::message_seen,
                payload:  serde_json::json!({
                    "conversation_id": message_seen.conversation_id,
                    "user_id": self.user_id,
                    "text_ids": message_seen.text_ids
                }),
            },
        });
    }

    fn handle_call_signal(&mut self, payload: Value) {
        let payload: IncomingSignal = match serde_json::from_value(payload) {
            Ok(p) => p,
            Err(error) => {
                log::error!("Invalid call signal: {:?}", error);
                return;
            }
        };

        let outgoing = self.build_outgoing_signal(&payload);

        let message = WsEnvelope {
            msg_type: WsEnvelopeType::call_signal,
            payload: outgoing,
        };

        match payload.r#type.as_str() {
            "offer" | "answer" | "ice_candidate" => {
                self.send_direct_message(
                    payload.to,
                    &payload.r#type,
                    message
                );
            }
            "call_request" | "call_accept" | "call_reject" | "call_end" => {
                self.send_direct_message(
                    payload.to,
                    &payload.r#type,
                    message
                );
            }
            "call_start" | "call_join" | "call_leave" | "video_toggle" | "audio_toggle" => {
                self.send_room_signal(
                    payload.room_id,
                    &payload.r#type,
                    message
                );
            }
            _ => {
                log::warn!("Unknown call signal type: {}", payload.r#type);
            }
        }
    }

    fn build_outgoing_signal(&self, payload: &IncomingSignal) -> serde_json::Value {
        let mut outgoing = serde_json::json!({
            "type": payload.r#type,
            "from": self.user_id,
        });

        if let Some(sdp) = &payload.sdp {
            outgoing["sdp"] = serde_json::Value::String(sdp.clone());
        }
        if let Some(candidate) = &payload.candidate {
            outgoing["candidate"] = candidate.clone();
        }
        if let Some(call_type) = &payload.call_type {
            outgoing["call_type"] = serde_json::Value::String(
                call_type.clone()
            );
        }
        if let Some(enabled)   = payload.enabled {
            outgoing["enabled"] = serde_json::Value::Bool(enabled);
        }
        if let Some(muted) = payload.muted {
            outgoing["muted"] = serde_json::Value::Bool(muted);
        }
        if let Some(room_id) = &payload.room_id {
            outgoing["room_id"] = serde_json::Value::String(
                room_id.clone()
            );
        }
        if let Some(to) = &payload.to {
            outgoing["to"]  = serde_json::Value::String(to.clone());
        }

        outgoing
    }

    fn send_direct_message(
        &self,
        to: Option<String>,
        signal_type: &str,
        message: WsEnvelope
    ) {
        match to {
            None => log::error!("{} missing 'to' field", signal_type),
            Some(to_user_id) => self.lobby_addr.do_send(DirectMessage {
                from_user_id: self.user_id.clone(),
                to_user_id,
                msg: message,
            }),
        }
    }

    fn send_room_signal(
        &self,
        room_id: Option<String>,
        signal_type: &str,
        message: WsEnvelope
    ) {
        match room_id {
            None => log::error!("{} missing 'room_id' field", signal_type),
            Some(room_id) => self.lobby_addr.do_send(RoomSignalMessage {
                from_user_id: self.user_id.clone(),
                room_id,
                msg: message,
            }),
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
    fn handle(
        &mut self,
        msg: Result<ws::Message,
        ws::ProtocolError>,
        ctx: &mut Self::Context
    ) {
    match msg {
        Ok(ws::Message::Ping(msg)) => {
            self.hb = Instant::now();
            ctx.pong(&msg);
        }
        Ok(ws::Message::Pong(_)) => {
            self.hb = Instant::now()
        },
        Ok(ws::Message::Binary(binary)) => {
            println!("Something in binary");
            ctx.binary(binary)
        },
        Ok(ws::Message::Close(reason)) => {
            ctx.close(reason);
            ctx.stop();
        },
        Ok(ws::Message::Continuation(_)) => {
            println!("Something in continuation");
            ctx.stop();
        },
        Ok(ws::Message::Nop) => (),
        Ok(ws::Message::Text(text)) => {
            self.handle_text_message(text.to_string());
        }
        Err(error) => {
            log::error!("Error: {:?}", error); ctx.stop();
        }
    }
  }
}

impl Handler<WsMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0)
    }
}

async fn add_message_seen(message_seen: MessageSeenPayload, user_id: &str) {
    let db = MongoDB.connect();

    let now = Utc::now().timestamp_millis();
    let mut messages_to_read = Vec::new();

    for text_id in message_seen.text_ids.clone() {   
        let message_read = Conversation::MessageRead {
            message_id: text_id,
            user_id: user_id.to_string(),
            read_at: now.clone(),
        };

        messages_to_read.push(message_read);
    }
    
    let collection = db.collection::<Conversation::MessageRead>("message_read");

    let result = collection.insert_many(messages_to_read).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        return;
    }
}

async fn save_message_in_database(message: SocketOutgoingTextModel) {
    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
        return;
    }

    // Update last message
    let collection = db.collection::<Conversation::ConversationCore>("conversation_core");
    let result = collection.update_one(
        doc!{"uuid": message.conversation_id.clone()},
        doc!{"$set": {
            "last_message_id": message.uuid.clone(),
            "last_message_at": message.created_at.clone(),
        }},
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return;
    }
  
    let message_core = Conversation::MessageCore {
        uuid: message.uuid.clone(),  
        conversation_id: message.conversation_id.clone(),
        owner: message.owner.clone(),
        reply_to: message.reply_to.clone(),
        created_at: message.created_at.clone(),
        r#type: message.r#type.clone(),
    };
        
    let collection = db.collection::<Conversation::MessageCore>("message_core");
    let result = collection.insert_one(
        &message_core,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return;
    }

    let message_content = Conversation::MessageContent {
        message_id: message.uuid.clone(),
        text: message.text.clone(),
        audio: match message.audio {
            None => None,
            Some(audio) => Some(audio.uuid.clone()),
        },
        video: message.video.clone(),
        images: message.images.clone(),
        attachment: match message.attachment {
            None => None,
            Some(attachment) => Some(attachment.uuid.clone()),
        },
        emoji: message.emoji.clone(),
        mentions: message.mentions.clone(),
    };

    let collection = db.collection::<Conversation::MessageContent>("message_content");
    let result = collection.insert_one(
        &message_content,
    ).await;
    
    if let Err(error) = result {
        log::error!("{:?}", error);
        session.abort_transaction().await.ok().unwrap();
        return;
    }

    /* DATABASE ACID COMMIT */
    let _ = session.commit_transaction().await;
}