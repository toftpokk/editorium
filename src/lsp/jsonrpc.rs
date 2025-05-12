// implement my own client since jsonrpc_lite moved
// to bitcoin library
// inspired by: https://github.com/karyontech/karyon/tree/master/jsonrpc

// ref: https://www.jsonrpc.org/specification
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    jsonrpc: String,
    pub id: Option<Value>,

    // request
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    // response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl Message {
    pub fn is_request_object(&self) -> bool {
        self.method.is_some()
    }

    pub fn is_request(&self) -> bool {
        self.method.is_some() && self.id.is_some()
    }

    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    pub fn is_response(&self) -> bool {
        self.result.is_some()
    }

    pub fn as_request(self) -> Request {
        Request {
            jsonrpc: self.jsonrpc,
            method: self.method.unwrap(),
            params: self.params,
            id: self.id.unwrap(),
        }
    }

    pub fn as_notification(self) -> Notification {
        Notification {
            jsonrpc: self.jsonrpc,
            method: self.method.unwrap(),
            params: self.params,
        }
    }

    pub fn as_response(self) -> Response {
        Response {
            result: self.result,
            error: self.error,
            id: self.id.unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    jsonrpc: String,
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    pub id: Value,
}

impl Request {
    pub fn new(id: u64, method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: serde_json::to_value(id).unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    jsonrpc: String,
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

pub fn request(id: u64, method: &str, params: Option<Value>) -> String {
    let request = Request::new(id, method, params);

    serde_json::to_string(&request).unwrap()
}

#[derive(Serialize, Deserialize, Debug)]

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
