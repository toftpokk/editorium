use lsp_types;
use serde_json::json;
use smol::{
    block_on,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
};
use std::{path::PathBuf, process::Stdio, str::FromStr, string::ParseError};

use super::jsonrpc;

#[derive(Debug)]
pub enum Error {
    URIParseError(ParseError),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

// connection to spawned lsp server
pub struct Connection {
    _process: Child, // store, so kill_on_drop does not drop
    stdout: BufReader<ChildStdout>,
    stdin: BufWriter<ChildStdin>,
    stderr: BufReader<ChildStderr>,

    last_request_id: u32,
}

impl Connection {
    fn new(
        process: Child,
        stdout: BufReader<ChildStdout>,
        stdin: BufWriter<ChildStdin>,
        stderr: BufReader<ChildStderr>,
    ) -> Self {
        Self {
            _process: process,
            stdout,
            stdin,
            stderr,
            last_request_id: 0,
        }
    }

    async fn send(&mut self, method: &str, params: Option<serde_json::Value>) -> u32 {
        self.last_request_id += 1;
        let request_id = self.last_request_id;
        let mut request = jsonrpc::request(request_id, method, params);

        let header = format!("Content-Length: {}\r\n\r\n", request.len());

        request.insert_str(0, &header);
        self.stdin.write_all(request.as_bytes()).await.unwrap();
        self.stdin.flush().await.unwrap();

        request_id
    }

    async fn recv(&mut self) -> Result<(serde_json::Value, serde_json::Value), Error> {
        let mut content_length = 0;
        let mut content_type = "application/vscode-jsonrpc; charset=utf-8".to_string();
        loop {
            let mut header_line = String::new();
            self.stdout.read_line(&mut header_line).await?;

            let header_line = header_line.as_str().trim();
            if header_line.len() == 0 {
                // end of headers
                break;
            }

            let parts = header_line.split_once(":").unwrap();
            let header = parts.0.trim();
            let value = parts.1.trim();
            match header {
                "Content-Length" => {
                    content_length = value.parse().unwrap();
                }
                "Content-Type" => {
                    content_type = value.to_string();
                }
                header => {
                    log::warn!("Unknown header: {}", header)
                }
            }
        }

        let mut buf = vec![0; content_length];
        self.stdout.read_exact(&mut buf).await?;

        if content_type != "application/vscode-jsonrpc; charset=utf-8" {
            panic!("Unknown content type: {}", content_type);
        }

        let content = String::from_utf8(buf).unwrap();

        let content_response = jsonrpc::response(&content);
        match content_response.error {
            Some(err) => panic!("Could not read response: {}", err),
            None => {}
        }

        let result = content_response.result.unwrap();

        Ok((result, content_response.id))
    }
}

pub enum ClientKind {
    None,
    Uninitialized,
    Initialized,
}

// impl ClientKind {
//     fn into_connection(&mut self) -> Option<&mut Connection> {
//         match self {
//             Self::Initialized => Some(conn),
//             Self::Uninitialized(conn) => Some(conn),
//             _ => None,
//         }
//     }
// }

pub struct Client {
    kind: ClientKind,
    file: Option<PathBuf>,
    connection: Option<Connection>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            kind: ClientKind::None,
            file: None,
            connection: None,
        }
    }

    pub fn connect(&mut self, file: PathBuf) -> Result<(), Error> {
        match self.kind {
            ClientKind::None => {}
            _ => panic!("only unconnected clients may connect"),
        }
        // note cat reads stdin and echos to stdout
        // good command for test
        let process = Command::new("gopls")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        let mut process = match process {
            Ok(process) => process,
            Err(e) => {
                panic!("{:?}", e)
            }
        };

        let stdout = BufReader::new(process.stdout.take().expect("Failed to open stdout"));
        let stdin = BufWriter::new(process.stdin.take().expect("Failed to open stdin"));
        let stderr = BufReader::new(process.stderr.take().expect("Failed to open stderr"));

        self.kind = ClientKind::Uninitialized;
        self.connection = Some(Connection::new(process, stdout, stdin, stderr));
        self.file = Some(file);

        Ok(())
    }

    pub fn initialize(&mut self) {
        match &self.kind {
            ClientKind::Uninitialized => {}
            _ => {
                panic!("should be in state 'uninitialized'");
            }
        }
        let connection = self.connection.as_mut().unwrap();

        let params = Self::init_params(self.file.as_ref().unwrap(), "test".to_string()).unwrap();

        let params = serde_json::to_value(params).unwrap();
        let req_id = block_on(connection.send("initialize", Some(params)));

        let (result, id) = block_on(connection.recv()).unwrap();
        // TODO out of order messages
        if id != req_id {
            panic!("message out of order")
        }

        self.kind = ClientKind::Initialized;

        println!("{:?}", result);
    }

    pub fn init_params(
        workspace: &PathBuf,
        workspace_name: String,
    ) -> Result<lsp_types::InitializeParams, Error> {
        let process_id = std::process::id();

        // TODO handle parse url error
        let workspace_uri = lsp_types::Uri::from_str(workspace.to_str().unwrap()).unwrap();
        let workspace_folder = lsp_types::WorkspaceFolder {
            uri: workspace_uri,
            name: workspace_name,
        };

        // TODO options for each lsp
        let options = json!({});

        Ok(lsp_types::InitializeParams {
            process_id: Some(process_id),
            workspace_folders: Some(vec![workspace_folder]),
            initialization_options: Some(options),
            capabilities: lsp_types::ClientCapabilities {
                workspace: Some(lsp_types::WorkspaceClientCapabilities {
                    apply_edit: Some(true), // modify resource
                    workspace_edit: Some(lsp_types::WorkspaceEditClientCapabilities {
                        document_changes: Some(true), // versioned document changes
                        resource_operations: Some(vec![
                            lsp_types::ResourceOperationKind::Create,
                            lsp_types::ResourceOperationKind::Delete,
                            lsp_types::ResourceOperationKind::Rename,
                        ]),
                        failure_handling: Some(lsp_types::FailureHandlingKind::Abort), // change is aborted if failed
                        normalizes_line_endings: Some(false),
                        change_annotation_support: None,
                    }),
                    did_change_configuration: Some(
                        lsp_types::DynamicRegistrationClientCapabilities {
                            dynamic_registration: Some(false),
                        },
                    ),
                    ..Default::default() // did_change_watch?
                }),
                ..Default::default() // text_document: (),
                                     // notebook_document: (),
                                     // window: (),
                                     // general: (),
                                     // experimental: (),
            },
            // trace: todo!(),
            // workspace_folders: todo!(),
            // client_info: todo!(),
            // locale: todo!(),
            // work_done_progress_params: todo!(),
            // deprecated
            root_path: None,
            root_uri: None,

            ..Default::default()
        })
    }
}
