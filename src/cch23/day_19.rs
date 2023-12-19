use std::str::FromStr;

use rocket::get;
use rocket_ws::{Message, Stream, WebSocket};

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

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![ws_ping]
}
