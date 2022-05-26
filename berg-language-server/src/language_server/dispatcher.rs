pub trait Dispatcher {
    fn on_request(&self, server: &Server, request: lsp_server::Request);
    fn on_notification(&self, server: &Server, request)
}

pub struct RawRequest {
    server: &Server,
    
}
pub trait Handler {
    fn handle_request(&self, ) -> Result<Response, Box<dyn Error+Sync+Send>>;
    fn handle_notification(&self, ) -> Result<Response, Box<dyn Error+Sync+Send>>;
}

