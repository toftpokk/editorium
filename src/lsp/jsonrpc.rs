// implement my own client since jsonrpc_lite moved
// to bitcoin library
// inspired by: https://github.com/karyontech/karyon/tree/master/jsonrpc

use std::fmt::Display;

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

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(r) = self.as_request() {
            write!(f, "{}", r)
        } else if let Some(r) = self.as_notification() {
            write!(f, "{}", r)
        } else if let Some(r) = self.as_response() {
            write!(f, "{}", r)
        } else {
            let s = self.clone();
            write!(
                f,
                "Message: id: {} method: {}\n{}\n{}\n{}",
                s.id.unwrap_or(Value::Null),
                s.method.unwrap_or("".to_string()),
                s.params.unwrap_or(Value::Null),
                s.result.unwrap_or(Value::Null),
                s.error.unwrap_or(Value::Null)
            )
        }
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl Message {
    // technically notifications are a subset of requests
    // here, they are separate with an umbrella term 'request_object' for both
    pub fn is_request_object(&self) -> bool {
        self.method.is_some()
    }

    pub fn to_string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn as_request(&self) -> Option<Request> {
        if self.method.is_some() && self.id.is_some() {
            Some(Request {
                jsonrpc: self.jsonrpc.clone(),
                method: self.method.clone().unwrap(),
                params: self.params.clone(),
                id: self.id.clone().unwrap(),
            })
        } else {
            None
        }
    }

    pub fn as_notification(&self) -> Option<Notification> {
        if self.method.is_some() && self.id.is_none() {
            Some(Notification {
                jsonrpc: self.jsonrpc.clone(),
                method: self.method.clone().unwrap(),
                params: self.params.clone(),
            })
        } else {
            None
        }
    }

    pub fn as_response(&self) -> Option<Response> {
        if self.result.is_some() {
            Some(Response {
                result: self.result.clone(),
                error: self.error.clone(),
                id: self.id.clone().unwrap(),
            })
        } else {
            None
        }
    }
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        Self {
            jsonrpc: Default::default(),
            id: Some(value.id),
            method: None,
            params: None,
            result: value.result,
            error: value.error,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    jsonrpc: String,
    pub method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    pub id: Value,
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(params) = &self.params {
            write!(
                f,
                "Request: with {} method {}\n{}",
                self.id, self.method, params
            )
        } else {
            write!(f, "Request: with {} method {}", self.id, self.method)
        }
    }
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

    pub fn to_string(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn as_message(self) -> Message {
        Message {
            jsonrpc: self.jsonrpc,
            id: Some(self.id),
            method: Some(self.method),
            params: self.params,
            result: None,
            error: None,
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

impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(params) = &self.params {
            write!(f, "Notification: method {}\n{}", self.method, params)
        } else {
            write!(f, "Notification: method {}", self.method)
        }
    }
}

impl Notification {
    pub fn new(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        }
    }

    pub fn as_message(self) -> Message {
        Message {
            jsonrpc: self.jsonrpc,
            id: None,
            method: Some(self.method),
            params: self.params,
            result: None,
            error: None,
        }
    }
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

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(error) = &self.error {
            write!(f, "Response: for {}\n{}", self.id, error)
        } else if let Some(result) = &self.result {
            write!(f, "Response: for {}\n{}", self.id, result)
        } else {
            write!(f, "Response: for {}", self.id)
        }
    }
}
