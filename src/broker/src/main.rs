use std::net::{SocketAddr, UdpSocket};

use coap::Server;
use coap_lite::CoapRequest;
use tokio::runtime::Runtime;
use coap_lite::{Packet, RequestType as Method, MessageClass};



fn main() {
    let addr = "127.0.0.1:5683";

    Runtime::new().unwrap().block_on(async move {
        let server = Server::new_udp(addr).unwrap();
        println!("Broker up on {}", addr);

        server
            .run(|mut request: Box<CoapRequest<SocketAddr>>| async move {
                match request.get_method() {
                    &Method::Get => println!("request by get {}", request.get_path()),
                    &Method::Post => println!(
                        "request by post {}",
                        String::from_utf8(request.message.payload.clone()).unwrap()
                    ),
                    &Method::Put => println!(
                        "request by put {}",
                        String::from_utf8(request.message.payload.clone()).unwrap()
                    ),
                    _ => println!("request by other method"),
                };

                match request.response {
                    Some(ref mut message) => {
                        message.message.payload = b"OK".to_vec();
                        let _ = inform_client("127.0.0.1:5684", request.message.payload.clone()).await;
                    }
                    _ => {}
                };
                return request;
            })
            .await
            .unwrap();
    });
}

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