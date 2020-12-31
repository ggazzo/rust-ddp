use uuid::Uuid;
use async_tungstenite::tungstenite::Message;

pub struct Client {
    pub id: Uuid,
    sender: crate::Tx<Message>,
}

impl Client {
    pub fn new(x: crate::Tx<Message>) -> Client {
        Client {
            id: Uuid::new_v4(),
            sender: x
        }
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
        match serde_json::from_str::<crate::ddp::MessageRequest>(msg) {
            Ok(message)  => match message {
                crate::ddp::MessageRequest::Ping { id } => self.pong(id),
                crate::ddp::MessageRequest::Pong { id } => self.ping(id),
                crate::ddp::MessageRequest::Method => println!("method"),
                crate::ddp::MessageRequest::Connect => println!("connected"),
                _ => { self.close(); println!("not implemented"); } ,
            },
            Err(e) => self.close(),
        }
    }
}

struct Timer ();

impl Timer {

}