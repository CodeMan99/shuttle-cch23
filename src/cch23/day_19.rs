use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use rocket::futures::{SinkExt, StreamExt};
use rocket::{get, post, State};
use rocket_ws::{Message, Stream, WebSocket};
use tokio::sync::mpsc::{self, error::SendError, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameState {
    Connected,
    Started,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameMessage {
    Serve,
    Ping,
}

struct GameMessageUnreconized;

impl FromStr for GameMessage {
    type Err = GameMessageUnreconized;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "serve" => Ok(GameMessage::Serve),
            "ping" => Ok(GameMessage::Ping),
            _ => Err(GameMessageUnreconized),
        }
    }
}

#[get("/ws/ping")]
fn ws_ping(ws: WebSocket) -> Stream!['static] {
    Stream! { ws =>
        let mut game_state = GameState::Connected;
        for await message in ws {
            if let Message::Text(text) = message? {
                match text.parse() {
                    Ok(GameMessage::Serve) => game_state = GameState::Started,
                    Ok(GameMessage::Ping) if game_state == GameState::Started => {
                        yield "pong".into();
                    },
                    _ => (),
                }
            }
        }
    }
}

pub mod bird_app {
    use super::*;
    use std::hash::Hash;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
    pub struct User {
        pub user: String,
    }

    impl User {
        pub fn new(user: &str) -> Self {
            User { user: user.into() }
        }

        pub fn enter_message(&self, raw: &RawMessage) -> Option<Message> {
            if raw.message.len() <= 128 {
                Some(Message {
                    user: self.clone(),
                    message: raw.message.clone(),
                })
            } else {
                None
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
    pub struct RawMessage {
        message: String,
    }

    #[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
    pub struct Message {
        #[serde(flatten)]
        user: User,
        message: String,
    }

    pub type MessageSender = UnboundedSender<rocket_ws::Message>;

    #[derive(Default)]
    pub struct Room {
        pub users: HashMap<User, MessageSender>,
    }

    impl Room {
        pub fn broadcast_message(
            &self,
            message: &Message,
        ) -> Result<(), SendError<rocket_ws::Message>> {
            let msg_text: String = serde_json::to_string(message).unwrap();

            for (_user, stream) in self.users.iter() {
                let message = rocket_ws::Message::text(&msg_text);
                stream.send(message)?;
            }

            Ok(())
        }
    }

    #[derive(Default)]
    pub struct App {
        pub rooms: Arc<RwLock<HashMap<usize, Room>>>,
        pub view_count: Arc<AtomicU64>,
    }

    pub fn create_app() -> App {
        App::default()
    }
}

#[post("/reset")]
async fn reset_bird_app(app: &State<bird_app::App>) {
    app.view_count.fetch_and(0, Ordering::Relaxed);
    let mut rooms = app.rooms.write().await;
    for (_, room) in rooms.iter_mut() {
        for (_, stream) in room.users.iter() {
            stream.downgrade();
        }
        room.users.clear();
    }
    rooms.clear();
}

#[get("/views")]
fn views(app: &State<bird_app::App>) -> String {
    app.view_count.fetch_max(0, Ordering::Relaxed).to_string()
}

#[get("/ws/room/<room_id>/user/<user>")]
fn ws_bird_room<'ws>(
    room_id: usize,
    user: &'ws str,
    app: &'ws State<bird_app::App>,
    ws: WebSocket,
) -> rocket_ws::Channel<'ws> {
    ws.channel(move |stream| {
        Box::pin(async move {
            let (mut sender, mut receiver) = stream.split();
            let (tx, mut rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
                mpsc::unbounded_channel();
            let view_count = app.view_count.clone();

            // this enables the broadcast
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    sender
                        .send(msg)
                        .await
                        .expect("Message Passing Failed rx => sender");
                    view_count.fetch_add(1, Ordering::Relaxed);
                }
                sender.close().await.unwrap();
            });

            let user = bird_app::User::new(user);

            // insert user into room
            {
                let mut rooms_lock = app.rooms.write().await;
                if let Some(room) = rooms_lock.get_mut(&room_id) {
                    room.users.insert(user.clone(), tx);
                } else {
                    let mut room = bird_app::Room::default();
                    room.users.insert(user.clone(), tx);
                    rooms_lock.insert(room_id, room);
                }
            }

            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    if let Ok(raw_message) = serde_json::from_str(&text) {
                        if let Some(msg) = user.enter_message(&raw_message) {
                            let rooms_lock = app.rooms.read().await;

                            if let Some(room) = rooms_lock.get(&room_id) {
                                room.broadcast_message(&msg)
                                    .expect("Failed to broadcast message");
                            }
                        }
                    }
                }
            }

            // remove from room
            {
                let mut rooms_lock = app.rooms.write().await;

                if let Some(room) = rooms_lock.get_mut(&room_id) {
                    room.users.remove(&user);
                }
            }

            Ok(())
        })
    })
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![ws_ping, reset_bird_app, views, ws_bird_room]
}

#[cfg(test)]
mod tests_day_19 {
    use super::*;

    /// Enforce traits needed for rocket to manage state of <T>
    fn is_manage_safe<T: Send + Sync + 'static>() {}

    #[test]
    fn test_app_is_manage_safe() {
        is_manage_safe::<bird_app::App>();
    }
}
