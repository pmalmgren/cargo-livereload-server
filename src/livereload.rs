use futures_util::{SinkExt, StreamExt};

use warp::ws::{Message, WebSocket};
use warp::Filter;

use serde::{Deserialize, Serialize};
// use serde_json::Result as SerdeResult;

#[derive(Serialize, Deserialize)]
struct ClientHandshake {
    command: String,
    protocols: Vec<String>,
    ver: String,
}

#[derive(Serialize, Deserialize)]
struct ServerHandshake {
    command: String,
    protocols: Vec<String>,
}

enum ClientState {
    Handshake(ClientHandshake),
    // optional: store information here about the client
    // Ready,
    Invalid,
}

async fn user_connected(ws: WebSocket) {
    let (mut wstx, mut wsrx) = ws.split();
    let result = wsrx.next().await;
    let state: ClientState = match result {
        Some(Ok(msg)) => match msg.to_str() {
            Err(_e) => ClientState::Invalid,
            Ok(msg_str) => match serde_json::from_str::<ClientHandshake>(msg_str) {
                Err(_e) => ClientState::Invalid,
                Ok(handshake) => ClientState::Handshake(handshake),
            },
        },
        None => {
            log::error!("Received null message");
            ClientState::Invalid
        }
        Some(Err(e)) => {
            log::error!("error receiving message: {}", e);
            ClientState::Invalid
        }
    };

    match state {
        ClientState::Handshake(client_handshake) => {
            if !client_handshake
                .protocols
                .contains(&"http://livereload.com/protocols-official-7".to_string())
            {
                log::error!("we can only handle version 7 of livereload");
                return;
            }
            let resp = ServerHandshake {
                command: "hello".to_string(),
                protocols: vec!["http://livereload.com/protocols-official-7".to_string()],
            };
            let resp_text = serde_json::to_string(&resp).unwrap();
            wstx.send(warp::ws::Message::text(resp_text.as_str()))
                .await
                .unwrap();
        }
        _ => {
            log::error!("error during livereload handshake, closing connection");
            return;
        }
    };

    while let Some(_msg) = wsrx.next().await {}
}

pub async fn server() {
    let notifyjs = warp::path("livereload.js").and(warp::fs::file("./static/livereload.js"));
    let testendpoint = warp::path("test").and(warp::fs::file("./static/test.html"));
    let notifyws = warp::path("livereload")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket: WebSocket| user_connected(socket)));
    let server = notifyjs.or(notifyws).or(testendpoint);

    warp::serve(server).run(([127, 0, 0, 1], 35729)).await;
}

mod test {
    use super::*;
    use warp;

    #[tokio::test]
    async fn test_it_handshakes() {
        let route = warp::ws()
            .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket: WebSocket| user_connected(socket)));

        let mut client = warp::test::ws()
            .path("livereload")
            .handshake(route)
            .await
            .expect("handshake");

        let handshake = ClientHandshake {
            command: "hello".to_string(),
            protocols: vec![
                "http://livereload.com/protocols-official-6".to_string(),
                "http://livereload.com/protocols-official-7".to_string(),
            ],
            ver: "3.3.2".to_string(),
        };
        client
            .send(Message::text(
                serde_json::to_string_pretty(&handshake).unwrap().as_str(),
            ))
            .await;
        let resp = client.recv().await.expect("recv");
        assert!(resp.is_text());
        let resp_handshake: ServerHandshake =
            serde_json::from_str(resp.to_str().unwrap()).expect("serialization error");

        assert_eq!(resp_handshake.command, "hello".to_string());
        assert_eq!(
            resp_handshake.protocols[0],
            "http://livereload.com/protocols-official-7".to_string()
        );
    }
}
