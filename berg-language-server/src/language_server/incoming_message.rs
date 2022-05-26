use lsp_server::RequestId;

impl<T: lsp_server::types::Request> IncomingMessage for T {

}


pub struct IncomingMessage<T: Request> {
    id: RequestId,
    params: T::Params
}

impl<T: Request> IncomingMessage<T> {
    pub type METHOD = T::METHOD;
    pub type Params = 
    pub fn method(&self) -> &'static str {
        return T::METHOD;
    }
}

// impl TryFrom<Message> for IncomingMessage {
//     type Error = Box<dyn Error+Sync+Send>;
//     fn try_from(message: Message) -> Result<Self, Self::Error> {
//         match message {
//             Message::Request(lsp_server::Request { id, method, params }) => match method.as_str() {
//                 _ => Err(ServerError(format!("Unexpected "))
//             }
//             Message::Notification(Notification { method, params }) => match method.as_str() {
//                 _ => Err(ServerError(format!("Unexpected notification {:?}", notification)))
//             }
//             Message::Response(response) => Err(ServerError(format!("Unexpected response {:?}", response)))
//         }
//     }
// }
