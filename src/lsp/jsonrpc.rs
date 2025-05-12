// implement my own client since jsonrpc_lite moved
// to bitcoin library
// inspired by: https://github.com/karyontech/karyon/tree/master/jsonrpc

// ref: https://www.jsonrpc.org/specification
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct InternalRequest<'a> {
    jsonrpc: String,
    method: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,

    id: Value,
}

#[derive(Serialize, Deserialize)]

pub struct Response {
    pub result: Option<Value>,
    pub error: Option<Value>,

    pub id: Value,
}

pub fn request(id: u32, method: &str, params: Option<Value>) -> String {
    let request = InternalRequest {
        jsonrpc: "2.0".to_string(),
        method,
        params,
        id: serde_json::to_value(id).unwrap(),
    };

    serde_json::to_string(&request).unwrap()
}

pub fn response(res: &str) -> Response {
    serde_json::from_str(res).unwrap()
}
