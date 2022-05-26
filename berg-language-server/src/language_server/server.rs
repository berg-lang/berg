use crossbeam_channel::{Sender, Receiver};
use lsp_server::{Connection, Message, ProtocolError, RequestId, Response, Notification, ResponseError};
use lsp_types::{notification::{Exit, Notification}, InitializeParams, ServerCapabilities};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::{error::Error, fmt::Display, collections::HashMap};

use super::Client;

/// A language server.
pub struct Server {
    /// The connection we're talking to the client over.
    connection: Connection,
    /// Initialize params from the client when the session was opened
    pub initialize_params: InitializeParams,
    handlers: HashMap<&'static str, Fn(Value)->Result<Value, ResponseError>>,
}


impl Server {
    fn run_stdio() -> Result<(), Box<dyn Error + Sync + Send>> {
        // Create the transport. Includes the stdio (stdin and stdout) versions but this could
        // also be implemented to use sockets or HTTP.
        let (connection, io_threads) = Connection::stdio();
        let server = Server::initialize(connection)?;
        server.main_loop()?;
        io_threads.join()?;
        Ok(())
    }

    fn main_loop(&self) -> Result<(), Box<dyn Error + Sync + Send>> {
        eprintln!("starting example main loop");
        for message in self.connection.receiver {
            eprintln!("got msg: {:?}", message);

            match message {
                Message::Request(request) => {
                    if self.connection.handle_shutdown(&request)? {
                        break;
                    }
                    if request.method.starts_with("$/") {
                        eprintln!("Ignored request: {:?}", request);
                    } else {
                        return ServerError(format!("Unimplemented request {:?}", request));
                    }
                }
                Message::Response(response) => {
                    return ServerError(format!("Unexpected response from client {:?}", response));
                }
                Message::Notification(notification) => {
                    if notification.method == Exit::METHOD {
                        return Err(ServerError("exit sent without shutdown first"));
                    }
                    // Ignore optional (unimplemented) messages that start with $/
                    if notification.method.starts_with("$/") {
                        eprintln!("Ignored notification: {:?}", notification);
                    } else {
                        return ServerError(format!("Unimplemented notification: {:?}", notification)),
                    }
                },
            }
        }
        Ok(())
    }

    fn connect(connection: Connection) -> Result<Server, ProtocolError> {
        let server_capabilities = serde_json::to_value(Server::server_capabilities()).unwrap();
        let client_capabilities = connection.initialize(server_capabilities)?;
        Ok(Server {
            connection,
            client_capabilities: serde_json::from_value(client_capabilities).unwrap(),
        })
    }

    fn run(&self, server_capabilities: &ServerCapabilities, server: &impl LanguageServer) -> Result<Server, ProtocolError> {

    }
}

pub trait ServerHandler {}

pub struct ServerError(pub String);
impl Error for ServerError {}
impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}
