use crossbeam_channel::Sender;
use lsp_server::{Message, RequestId, Response, ResponseError};
use serde::Serialize;

pub struct Client {
    sender: Sender<Message>,
}

impl Client {
    pub fn new(sender: Sender<Message>) -> Client {
        Client { sender }
    }
    pub(crate) fn notify<T: Serialize>(&self, method: &str, params: T) {
        let params = serde_json::to_value(params).unwrap();
        self.send(lsp_server::Notification { method, params });
    }
    pub(crate) fn respond<T: Serialize>(&self, id: RequestId, result: Result<T, ResponseError>) {
        let result = result
            .ok()
            .map(|result| serde_json::to_value(result).unwrap());
        let error = result.err();
        self.send(Response { id, result, error }.into());
    }
    fn send(&self, message: impl Into<Message>) {
        self.sender.send(message).unwrap();
    }
}
