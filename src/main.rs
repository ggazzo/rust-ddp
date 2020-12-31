use log::{info, debug};

mod ddp;
mod client;

use std::{
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio;

use futures::prelude::*;
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future, pin_mut,
};

use async_std::net::{TcpListener, TcpStream};
use tokio::task;
use async_tungstenite::tungstenite::Message;

type Tx<T> = Arc<Mutex<UnboundedSender<T>>>;

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    let _ = raw_stream.set_nodelay(true);

    let stream = async_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    
        debug!("WebSocket connection established: {}", addr);
    
    let (write, read) = stream.split();
    
    let (tx, rx) = unbounded::<Message>();

    let client = Arc::new(client::Client::new(Arc::new(Mutex::new(tx))));

    let c = client.clone();

    let broadcast_incoming = read
        .try_for_each(move |msg| {
            let msg_raw = msg.to_text().unwrap();
            c.handle(msg_raw);
            future::ok(())
        });

    let receive_from_others = rx.map(Ok).forward(write);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    debug!("{} disconnected", &addr);
}

async fn run() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        task::spawn(handle_connection(stream, addr));
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    run().await
}
