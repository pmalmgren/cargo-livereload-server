use futures_util::{FutureExt, StreamExt};

use warp::ws::WebSocket;
use warp::Filter;

async fn user_connected(ws: WebSocket) {
    let (tx, rx) = ws.split();
    rx.forward(tx)
        .map(|result| {
            if let Err(e) = result {
                log::error!("Error sending: {}", e);
            }
        })
        .await;
}

pub async fn server() {
    let notifyjs = warp::path("livereload.js").and(warp::fs::file("./static/livereload.js"));
    let testendpoint = warp::path("test").and(warp::fs::file("./static/test.html"));
    let notifyws = warp::path::end()
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(move |socket: WebSocket| user_connected(socket)));
    let server = notifyjs.or(notifyws).or(testendpoint);

    warp::serve(server).run(([127, 0, 0, 1], 35729)).await;
}
