use coap_lite::{RequestType as Method, CoapRequest};
use coap::Server;
use tokio::runtime::Runtime;
use std::net::SocketAddr;
mod resource;
use resource::Topic;
use resource::TopicCollection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct Subscriber {
    addr: SocketAddr,
}

type TopicMap = Arc<Mutex<HashMap<String, Topic>>>;

fn handle_get(req:&Box<CoapRequest<SocketAddr>>){
    //handle discovery, subscribe
}

fn handle_put(req:&Box<CoapRequest<SocketAddr>>){
    // handle publish
}

fn handle_post(req:&Box<CoapRequest<SocketAddr>>){
    // handle topic config etc
}

fn handle_delete(req:&Box<CoapRequest<SocketAddr>>){
    // handle deletion of topic
}

fn main() {
    let topics: TopicMap = Arc::new(Mutex::new(HashMap::new()));
    let addr = "127.0.0.1:5683";
    Runtime::new().unwrap().block_on(async move {
        let mut server = Server::new_udp(addr).unwrap();
        println!("Server up on {}", addr);

        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            
            match request.get_method() {
                &Method::Get => handle_get(&request),
                &Method::Post => handle_post(&request),
                &Method::Put => handle_put(&request),
                &Method::Delete => handle_delete(&request),
                _ => println!("request by other method"),
            };

            // placeholder response by server to client
            match request.response {
                Some(ref mut message) => {
                    message.message.payload = b"Request received by server".to_vec();
                },
                _ => {}
            };

            // respond to request
            return request;
        }).await.unwrap();
    });
}