// implement my own client since jsonrpc_lite moved
// to bitcoin library
// inspired by: https://github.com/karyontech/karyon/tree/master/jsonrpc

// ref: https://www.jsonrpc.org/specification
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct RawRequest {
    pub jsonrpc: String,
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    pub id: Option<Value>,
}

impl From<String> for RawRequest {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl RawRequest {
    fn as_request(self) -> Request {
        Request {
            jsonrpc: self.jsonrpc,
            method: self.method,
            params: self.params,
            id: self.id.unwrap(),
        }
    }

    fn as_notification(self) -> Notification {
        Notification {
            jsonrpc: self.jsonrpc,
            method: self.method,
            params: self.params,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    jsonrpc: String,
    method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,

    id: Value,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    jsonrpc: String,
    method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

pub fn request(id: u32, method: &str, params: Option<Value>) -> String {
    let request = Request {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: serde_json::to_value(id).unwrap(),
    };

    serde_json::to_string(&request).unwrap()
}

#[derive(Serialize, Deserialize)]

pub struct Response {
    pub result: Option<Value>,
    pub error: Option<Value>,

    pub id: Value,
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}
