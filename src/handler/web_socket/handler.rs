use uuid::Uuid;
use chrono::Utc;
use super::Lobby::Lobby;
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use crate::builtins::mongo::MongoDB;
use serde::{Deserialize, Serialize};
use super::WsMessage::{ClientActorMessage, Connect, Disconnect, WsMessage};

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
  WrapFuture
};

use crate::Model::{
  Conversation,
  ImageStruct,
  AudioStruct,
  VideoStruct,
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
  video: Option<VideoStruct>,
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
  video: Option<VideoStruct>,
  attachment: Option<AttachmentStruct>,
  emoji: Option<String>,
  r#type: Conversation::TextType,
  reply_to: Option<String>,
  seen_by: Vec<String>,
  created_at: i64,
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
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConn {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
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
        println!("Something in text");
        let str = text.to_string();
        let arr: Vec<&str> = str.split("::").collect::<Vec<&str>>();

        if arr.len() < 2 {
          ctx.stop();
          return;
        }

        if arr[0] == "%text%" {
            let incoming_text: Result<SocketIncomingTextModel, serde_json::Error> = serde_json::from_str(&arr[1]);
          
            if let Err(error) = incoming_text {
                log::error!("{:?}", error);
                ctx.stop();
                return;
            }
  
            let incoming_text = incoming_text.unwrap();
  
            let room_id = incoming_text.conversation_id.clone();

            let outgoing_message = SocketOutgoingTextModel {
                uuid: Uuid::new_v4().to_string(),
                owner: self.user_id.clone(),
                conversation_id: room_id.clone(),
                text: incoming_text.text.clone(),
                emoji: incoming_text.emoji.clone(),
                mentions: incoming_text.mentions.clone(),
                images: incoming_text.images.clone(),
                audio: incoming_text.audio.clone(),
                video: incoming_text.video.clone(),
                reply_to: incoming_text.reply_to.clone(),
                seen_by: vec![],
                created_at: Utc::now().timestamp_millis(),
                r#type: incoming_text.r#type,
                attachment: incoming_text.attachment.clone(),
            };

            let outgoing_message_clone = outgoing_message.clone();

            actix::spawn(async move {
                save_message_in_database(outgoing_message_clone).await;
            });
  
            let message = serde_json::to_string(&outgoing_message).unwrap();
            let message = format!("%text%::{}", message);
            self.lobby_addr.do_send(ClientActorMessage {
                user_id: self.user_id.clone(),
                room_id,
                msg: message,
            });
        }
        else if arr[0] == "%typing%" {
            self.lobby_addr.do_send(ClientActorMessage {
                user_id: self.user_id.clone(),
                room_id: arr[1].to_string(),
                msg: str.to_string(),
            });
        }
       },
      Err(error) => {
            log::error!("Error: {:?}", error);
            ctx.stop()
      },
    }
  }
}

impl Handler<WsMessage> for WsConn {
  type Result = ();

  fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
    ctx.text(msg.0)
  }
}

async fn save_message_in_database(message: SocketOutgoingTextModel) {
    /* DATABASE ACID SESSION INIT */
    let (db, mut session) = MongoDB.connect_acid().await;
    if let Err(error) = session.start_transaction().await {
        log::error!("{:?}", error);
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
        video: match message.video {
            None => None,
            Some(video) => Some(video.uuid.clone()),
        },
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