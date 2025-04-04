use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt}; // Required for `split` and `send()`
use warp::ws::{Message, WebSocket};
use warp::Filter;

pub async fn start_websocket_server(tx: broadcast::Sender<Vec<u8>>) {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx.clone();
            ws.on_upgrade(move |socket| handle_connection(socket, tx))
        });

    println!("WebSocket server running on ws://127.0.0.1:3030/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}

pub async fn handle_connection(ws: WebSocket, tx: broadcast::Sender<Vec<u8>>) {
    let (mut ws_sender, mut ws_receiver) = ws.split(); // Corrected: Now ws_sender is defined
    let mut rx = tx.subscribe();

    // Fix: Using ws_sender instead of the undefined sender
    tokio::spawn(async move {
        while let Ok(img) = rx.recv().await {
            if let Err(e) = ws_sender.send(Message::binary(img)).await {
                eprintln!("WebSocket send error: {}", e);
                break; // Stop if send fails
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        if let Ok(msg) = result {
            println!("Received: {:?}", msg);
        }
    }
}
