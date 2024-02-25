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


//let _ = inform_client("127.0.0.1:5684", request.message.payload.clone()).await;
/*
async fn inform_client(client_addr: &str, payload: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let client_addr: SocketAddr = client_addr.parse()?;
    
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(client_addr)?;

    let mut packet = Packet::new();
    packet.header.set_type(coap_lite::MessageType::Confirmable);
    packet.header.code = MessageClass::Request(Method::Post); 
    packet.payload = payload.clone();

    let message = packet.to_bytes()?;
    socket.send(&message)?;

    let mut buffer = [0; 1024];
    let (size, _) = socket.recv_from(&mut buffer)?;
    let response_packet = Packet::from_bytes(&buffer[..size])?;
    println!("Received response: {:?}", response_packet);

    Ok(())

}
*/