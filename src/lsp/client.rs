use lsp_types::{self, request::Request};
use serde_json::json;
use smol::{
    Task, block_on,
    channel::{self, Receiver},
    future,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    spawn,
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

pub enum Message {
    InitializeResult,
    UnknownRequest(String),
}

// connection to spawned lsp server
// pub struct Connection {
//     _process: Child, // store, so kill_on_drop does not drop
//     stdout: BufReader<ChildStdout>,
//     stdin: BufWriter<ChildStdin>,
//     stderr: BufReader<ChildStderr>,

//     last_request_id: u32,
// }

// impl Connection {
//     fn new(
//         process: Child,
//         stdout: BufReader<ChildStdout>,
//         stdin: BufWriter<ChildStdin>,
//         stderr: BufReader<ChildStderr>,
//     ) -> Self {
//         Self {
//             _process: process,
//             stdout,
//             stdin,
//             stderr,
//             last_request_id: 0,
//         }
//     }

//     // poll a message
//     async fn poll_message(&mut self) -> Result<Message, Error> {
//         let message = self.recv().await?;

//         if message.is_response() {
//             // Note: when requesting something, client should block & wait for response
//             // TODO: poller should be able to push responses to caller
//             panic!("response should not be polled: {:?}", message)
//         }

//         if message.is_notification() {
//             let notification = message.as_notification();
//             return Ok(Message::UnknownRequest(notification.method));
//         }
//         if message.is_request() {
//             let request = message.as_request();
//             return Ok(Message::UnknownRequest(request.method));
//         }

//         panic!("unknown message: {:?}", message);
//     }
// }

pub enum ClientKind {
    None,
    Uninitialized,
    Initialized,
}

pub struct Client {
    pub kind: ClientKind,
    pub file: Option<PathBuf>,
    pub transport: Option<Transport>,
    pub server_capabilities: Option<lsp_types::ServerCapabilities>,
    pub stdin: Option<BufWriter<ChildStdin>>,
    pub stdout: Option<BufReader<ChildStdout>>,

    _write_worker: Option<channel::Receiver<String>>, // single instance
    _writer: Option<channel::Sender<String>>,         // cloned

    _reader_worker: Option<channel::Sender<String>>, // single instance
    _reader: Option<channel::Receiver<String>>,      // cloned
}

// so that Task::perform can clone only Writer, not whole client
pub struct Writer {
    writer: channel::Sender<String>,
}

impl Writer {
    pub async fn write(&self, msg: String) -> Result<(), channel::SendError<String>> {
        self.writer.send(msg).await
    }
}

// so that lsp_worker can clone only writer
pub struct Worker {}

impl Client {
    pub fn new() -> Self {
        Self {
            kind: ClientKind::None,
            file: None,
            transport: None,
            server_capabilities: None,

            stdin: None,
            stdout: None,
            _write_worker: None,
            _writer: None,
            _reader_worker: None,
            _reader: None,
        }
    }

    pub async fn worker(&mut self) {
        let obj = self._write_worker.as_ref().unwrap().recv().await.unwrap();
        self.stdin
            .as_mut()
            .unwrap()
            .write(obj.as_bytes())
            .await
            .unwrap();
        // smol::future::race(
        //     async {
        //         let obj = self._write_worker.unwrap().recv().await.unwrap();
        //         self.stdin.unwrap().write(obj.as_bytes());
        //     },
        //     async {
        //         let m
        //         self.stdout.unwrap().read();
        //         self._reader_worker.unwrap().send(msg).await;
        //     },
        // )
        // .await
    }

    pub fn new_writer(&self) -> Writer {
        let w = self._writer.clone();
        Writer { writer: w.unwrap() }
    }

    pub fn connect(&mut self, file: PathBuf) -> Result<(), Error> {
        match self.kind {
            ClientKind::None => {}
            _ => panic!("only unconnected clients may connect"),
        }
        // note cat reads stdin and echos to stdout
        // good command for test
        let process = Command::new("rust-analyzer")
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
        // let stdout = BufReader::new(process.stdout.take().expect("Failed to open stdout"));
        self.stdin = Some(BufWriter::new(
            process.stdin.take().expect("Failed to open stdin"),
        ));
        let (send, recv) = channel::unbounded::<String>();
        self._writer = Some(send);
        self._write_worker = Some(recv);

        self.stdout = Some(BufReader::new(
            process.stdout.take().expect("Failed to open stdout"),
        ));
        let (send, recv) = channel::unbounded::<String>();
        self._reader = Some(recv);
        self._reader_worker = Some(send);
        // let stderr = BufReader::new(process.stderr.take().expect("Failed to open stderr"));

        self.kind = ClientKind::Uninitialized;
        // self.transport = Some(Transport::new(stdin, stdout, stderr));
        self.file = Some(file);

        Ok(())
    }

    // pub fn new_transport_receiver(&self) -> Option<TransportReceiver> {
    //     match &self.kind {
    //         ClientKind::Initialized => {}
    //         _ => {
    //             return None;
    //         }
    //     }
    //     let receiver = self.transport.as_ref().unwrap().from_server.clone();
    //     Some(receiver.into())
    // }

    // async fn send(){
    //     let mut request = jsonrpc::request(request_id, method, params);

    //     let header = format!("Content-Length: {}\r\n\r\n", request.len());

    //     request.insert_str(0, &header);
    //     writer.write_all(request.as_bytes()).await.unwrap();
    //     writer.flush().await.unwrap();
    // }
    // pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
    //     if let Some(c) = &mut self.child {
    //         if let Some(stdout) = &mut c.stdout {
    //             return stdout.read(buf).await;
    //         }
    //     }
    //     Ok(0)
    // }

    pub fn initialize(&mut self) -> Option<String> {
        match &self.kind {
            ClientKind::Uninitialized => {}
            _ => {
                panic!("should be in state 'uninitialized'");
            }
        }
        if let Some(stdin) = &mut self.stdin {
            let params =
                Self::init_params(self.file.as_ref().unwrap(), "test".to_string()).unwrap();

            let client_capabilities = serde_json::to_value(params).unwrap();

            let mut request = jsonrpc::request(
                0,
                lsp_types::request::Initialize::METHOD,
                Some(client_capabilities),
            );
            let header = format!("Content-Length: {}\r\n\r\n", request.len());

            request.insert_str(0, &header);

            return Some(request);
            // stdin.write_all(request.as_bytes()).await.unwrap();
            // stdin.flush().await.unwrap();
        }
        None

        // let transport = self.transport.as_mut().unwrap();

        // // adds a new channel sender
        // let sender = transport.to_server.clone();
        // sender
        //     .send(jsonrpc::Request::new(
        //         0,
        //         lsp_types::request::Initialize::METHOD,
        //         Some(client_capabilities),
        //     ))
        //     .await
        //     .unwrap()
        // TODO handle request in task, not here
        // .unwrap();

        // let receiver = transport.from_server.clone();
        // // TODO: this recv may die
        // let response = receiver.recv().await.unwrap();
        // log::error!("what2?");

        // if !response.is_response() {
        //     panic!("message out of order: {:?}", response)
        // }
        // let response = response.as_response();
        // // TODO handle out of order
        // if response.id != 0 {
        //     panic!("out of order: {:?}", response)
        // }
        // if let Some(err) = response.error {
        //     panic!("initialization error: {}", err)
        // }
        // let initialize_result: lsp_types::InitializeResult =
        //     serde_json::from_value(response.result.unwrap()).unwrap();

        // // initialize_result.server_info
        // if let Some(server_info) = initialize_result.server_info {
        //     // Note: server version is too detailed
        //     log::info!("Connected to LSP server: {}", server_info.name)
        // } else {
        //     log::info!("Connected to LSP server: unknown server name")
        // }

        // self.server_capabilities = Some(initialize_result.capabilities);
        // self.kind = ClientKind::Initialized;
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

// a set of message channels for reading & writing to buffer
// instead of directly r/w to buffer
pub struct Transport {
    from_server: channel::Receiver<jsonrpc::Message>,
    _from_server_worker: Task<()>,
    to_server: channel::Sender<jsonrpc::Request>,
    _to_server_worker: Task<()>,
}

impl Transport {
    fn new(
        stdin: BufWriter<ChildStdin>,
        stdout: BufReader<ChildStdout>,
        stderr: BufReader<ChildStderr>,
    ) -> Self {
        // from lsp server
        let (chan_writer, from_server) = channel::unbounded::<jsonrpc::Message>();
        let from_server_worker = spawn(Self::from_server_worker(chan_writer, stdout, stderr));

        // to lsp server
        let (to_server, chan_reader) = channel::unbounded::<jsonrpc::Request>();
        let to_server_worker = spawn(Self::to_server_worker(chan_reader, stdin));

        // spawn(future)
        Self {
            from_server,
            _from_server_worker: from_server_worker,
            to_server,
            _to_server_worker: to_server_worker,
        }
    }

    // workers

    async fn from_server_worker(
        chan_writer: channel::Sender<jsonrpc::Message>,
        stdout: BufReader<ChildStdout>,
        stderr: BufReader<ChildStderr>,
    ) {
        // make it so stdxxx mutable
        let mut stdout = stdout;
        let mut stderr = stderr;
        loop {
            // receive message from both
            smol::future::race(
                Self::read_from_buf(&chan_writer, &mut stdout),
                Self::read_from_buf(&chan_writer, &mut stderr),
            )
            .await;
        }
    }

    async fn to_server_worker(
        chan_reader: channel::Receiver<jsonrpc::Request>,
        stdin: BufWriter<ChildStdin>,
    ) {
        let mut stdin = stdin; // make it so stdin mutable
        loop {
            // waits for message from channel
            let msg = chan_reader.recv().await;
            if let Ok(msg) = msg {
                Self::send(
                    msg.id.as_u64().unwrap(),
                    &msg.method,
                    msg.params,
                    &mut stdin,
                )
                .await;
            }
        }
    }

    // helper

    async fn read_from_buf<T>(
        chan_writer: &channel::Sender<jsonrpc::Message>,
        buf: &mut BufReader<T>,
    ) where
        T: smol::io::AsyncRead + Unpin, // need to be able to read async
    {
        let result = Self::recv(buf).await.unwrap();
        if let Some(result) = result {
            chan_writer.send(result).await.unwrap();
        }
    }

    async fn send<T>(
        request_id: u64,
        method: &str,
        params: Option<serde_json::Value>,
        writer: &mut BufWriter<T>,
    ) where
        T: smol::io::AsyncWrite + Unpin, // need to be able to read async
    {
        let mut request = jsonrpc::request(request_id, method, params);

        let header = format!("Content-Length: {}\r\n\r\n", request.len());

        request.insert_str(0, &header);
        writer.write_all(request.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }

    // recieve jsonrpc payload
    // note: AsyncRead polls, does not wait
    // maybe optimize?
    async fn recv<T>(reader: &mut BufReader<T>) -> Result<Option<jsonrpc::Message>, Error>
    where
        T: smol::io::AsyncRead + Unpin, // need to be able to read async
    {
        let mut content_length = 0;
        let mut content_type = "application/vscode-jsonrpc; charset=utf-8".to_string();
        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line).await?;
            // read_line will return empty string while polling
            if header_line.len() < 1 {
                return Ok(None);
            }

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
        reader.read_exact(&mut buf).await?;

        if content_type != "application/vscode-jsonrpc; charset=utf-8" {
            panic!("Unknown content type: {}", content_type);
        }

        let payload = String::from_utf8(buf).unwrap();

        Ok(Some(jsonrpc::Message::from(payload)))
    }
}

// receive server messages
pub struct TransportReceiver(Receiver<jsonrpc::Message>);

impl TransportReceiver {
    pub async fn recv(&self) -> jsonrpc::Message {
        self.0.recv().await.unwrap()
    }
}

impl From<Receiver<jsonrpc::Message>> for TransportReceiver {
    fn from(value: Receiver<jsonrpc::Message>) -> Self {
        TransportReceiver(value)
    }
}

// TODO use treesitter https://github.com/nvim-treesitter/nvim-treesitter?tab=readme-ov-file#supported-languages
