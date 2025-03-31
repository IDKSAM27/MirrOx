use warp::Filter;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use std::sync::Arc;

pub async fn start_websocket_server(tx: Arc<mpsc::UnboundedSender<Vec<u8>>>) {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| handle_connection(socket, tx))
        });

    println!("WebSocket server running on ws://127.0.0.1:3030/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket, tx: Arc<mpsc::UnboundedSender<Vec<u8>>>) {
    let (mut sender, _) = ws.split();

    let mut rx = tx.subscribe();

    while let Ok(img) = rs.recv().await {
        if let Err(e) = sender.send(warp::ws::Message::binary(img)).await {
            eprintln!("Failed to send image: {}", e);
            break;
        }
    }
}