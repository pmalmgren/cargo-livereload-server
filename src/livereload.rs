use futures_util::StreamExt;

use warp::ws::WebSocket;
use warp::Filter;

async fn user_connected(ws: WebSocket) {
    let (_wstx, mut wsrx) = ws.split();

    while let Some(result) = wsrx.next().await {
        if let Ok(msg) = result {
            log::info!("Received message: {:?}", msg);
        }
    }
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
    #[test]
    fn test_it_handshakes() {
    }
}
