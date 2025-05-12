use core::num;
use lsp_types;
use serde_json::json;
use smol::{
    block_on,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
};
use std::{path::PathBuf, process::Stdio, str::FromStr, string::ParseError};

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
    process: Child, // store, so kill_on_drop does not drop
    pub stdout: BufReader<ChildStdout>,
    pub stdin: BufWriter<ChildStdin>,
    stderr: BufReader<ChildStderr>,
}

pub struct Client {
    // Initialized(Connection),
    // Uninitialized,
    // None,
    pub connection: Option<Connection>,
}

impl Client {
    pub fn new(file: &PathBuf) -> Self {
        Self { connection: None }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        // note cat reads stdin and echos to stdout
        // good command for test
        let mut process = Command::new("cat")
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

        // process read buffers
        // let mut stdout = BufReader::new(process.stdout.take().expect("Failed to open stdout"));
        let mut stdin = BufWriter::new(process.stdin.take().expect("Failed to open stdin"));
        // let stderr = BufReader::new(process.stderr.take().expect("Failed to open stderr"));

        let buf = "hello\n";
        let num_b = block_on(stdin.write(buf.as_bytes())).unwrap();
        println!("{}", num_b);
        block_on(stdin.flush()).unwrap();

        let mut buf = vec![0; 1024];
        block_on(process.stdout.unwrap().read(&mut buf)).unwrap();

        let buf2 = String::from_utf8(buf).unwrap();
        println!("{}", buf2);

        // stdin.write_all(buf)
        // self.connection = Some(Connection {
        //     process,
        //     stdin,
        //     stdout,
        //     stderr,
        // });

        Ok(())
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
                    ..Default::default() // did_change_watch@2
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
