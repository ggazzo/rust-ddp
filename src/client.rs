use std::{
    sync::{Arc},
};

use uuid::Uuid;


use async_tungstenite::tungstenite::Message;

use tokio::sync::Notify;
use tokio::time::{timeout, Duration};
use tokio::task;

pub struct Client {
    pub id: Uuid,
    sender: crate::Tx<Message>,
    notify: Arc<Notify>,
}

impl Client {
    pub fn new(x: crate::Tx<Message>) -> Arc<Client> {

        let duration = Duration::from_millis(10000);  


        let notify = Arc::new(Notify::new());

        let client = Arc::new(Client {
            id: Uuid::new_v4(),
            sender: x,
            notify: notify.clone(),
        });

        let ret = client.clone();

        task::spawn(async move {
            while let Ok(_) = timeout(duration, notify.notified()).await {
                println!("did not receive value within 10 ms");
            }
            client.close();
        });  

        return ret

    }

    fn send(&self, message: Message) {
        self.sender.lock().unwrap().unbounded_send(message).unwrap();
    }
    
    pub fn close(&self) {
        self.send(Message::Close(None));
    }
    
    pub fn ping(&self, id: Option<String>) {
        self.send(Message::Text(crate::ddp::Ping::text(id)));
    }

    pub fn pong(&self, id: Option<String>) {
        self.send(Message::Text(crate::ddp::Pong::text(id)));
    }

    pub fn handle(&self, msg: &str) {
        self.notify.notify_one();
        match serde_json::from_str::<crate::ddp::MessageRequest>(msg) {
            Ok(message)  => match message {
                crate::ddp::MessageRequest::Ping { id } => self.pong(id),
                crate::ddp::MessageRequest::Pong { id } => self.ping(id),
                crate::ddp::MessageRequest::Method => println!("method"),
                crate::ddp::MessageRequest::Connect => println!("connected"),
                _ => { self.close(); println!("not implemented"); } ,
            },
            Err(_) => self.close(),
        }
    }
}