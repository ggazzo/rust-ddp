use serde::{Deserialize, Serialize};
use serde_json::json;

pub const VERSIONS: &'static [&'static str; 3] = &["1", "pre2", "pre1"];
pub type Ejson = serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "msg", rename_all = "lowercase" )]

/***************************
 *        requests        *
 ***************************/
pub enum MessageRequest {
    Connect,
    Ping { id: Option<String> },
    Pong { id: Option<String> },
    Method,
    Sub,
}

#[derive(Deserialize, Serialize)]
pub struct Connect {
    pub msg: String,
    pub session: String,
}

#[derive(Deserialize, Serialize)]
pub struct Failed {
    pub msg: String,
    pub version: String,
}

#[derive(Deserialize, Serialize)]
pub struct Ping {
}

#[derive(Deserialize, Serialize)]
pub struct Pong {
    pub msg: String,
    pub id:  Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Method {
    pub msg:    String,
    pub id:     String,
    pub result: String,
}

#[derive(Deserialize, Serialize)]
pub struct Sub {
    pub msg:    String,
    pub id:     String,
    pub result: String,
}

/***************************
 *        Requests         *
 ***************************/

impl Pong {
    pub fn text(id: Option<String>) -> String {
        if let Some(id) = id {
            json!({
                "msg": "pong",
                "id": id
            }).to_string()
        } else {
             json!({
                "msg": "pong"
            }).to_string()
        }
    }
}

impl Ping {
    pub fn text(id: Option<String>) -> String {
        if let Some(id) = id {
            json!({
                "msg": "ping",
                "id": id
            }).to_string()
        } else {
             json!({
                "msg": "ping"
            }).to_string()
        }
    }
}

impl Method {
    pub fn text<'l>(id: &'l str, method: &'l str, params: Option<&Vec<&Ejson>>) -> String {
        if let Some(args) = params {
             json!({
                "msg": "method",
                "id": id,
                "method": method,
                "params": args
            }).to_string()
        } else {
            json!({
                "msg": "method",
                "id": id,
                "method": method
            }).to_string()
        }
    }
}