mod messages;

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures::prelude::*;
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future, pin_mut,
};

use async_std::net::{TcpListener, TcpStream};
use async_std::task;
use async_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = async_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming
        .try_filter(|msg| {
            // Broadcasting a Close message from one client
            // will close the other clients.
            future::ready(!msg.is_close())
        })
        .try_for_each(|msg| {

            let msg_raw = msg.to_text().unwrap();

            println!(
                "Received a message from {}: {}",
                addr,
                msg_raw,
            );

            match serde_json::from_str::<messages::MessageRequest>(msg_raw) {
                Ok(message)  => match message {
                    messages::MessageRequest::Ping { id } => println!("{}", messages::Pong::text(id)),
                    messages::MessageRequest::Pong { id } => println!("{}", messages::Ping::text(id)),
                    messages::MessageRequest::Method => println!("method"),
                    messages::MessageRequest::Connect => println!("connected"),
                    _ => println!("not implemented"),
                },
                Err(e) => {println!("{}", e)},
            }

            future::ok(())
        });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

async fn run() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}

fn main() -> Result<(), IoError> {
    task::block_on(run())
}
