use serde_json::json;
use std::net::{SocketAddr, UdpSocket};
use coap_lite::{Packet, RequestType as Method, CoapOption, MessageClass};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::runtime::Runtime;

fn main() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        let broker_addr = "127.0.0.1:5683";
        let socket_addr = "0.0.0.0:0";

        loop {
            // Async read line from stdin
            let mut reader = BufReader::new(io::stdin()).lines();
            println!("Enter new temperature value: ");
            if let Ok(Some(line)) = reader.next_line().await {
                let temperature: i32 = match line.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Please enter a valid integer.");
                        continue;
                    },
                };

                match update_temperature(broker_addr, socket_addr, temperature).await {
                    Ok(_) => println!("Temperature updated successfully."),
                    Err(e) => println!("Failed to update temperature: {}", e),
                }
            }
        }
    });
}

async fn update_temperature(broker_addr: &str, socket_addr: &str, temperature: i32) -> Result<(), Box<dyn std::error::Error>> {
    let broker_addr: SocketAddr = broker_addr.parse()?;
    
    let socket = UdpSocket::bind(socket_addr)?;
    socket.connect(broker_addr)?;

    let path = "temperature/data"; 
    let payload = json!({"temperature": temperature}).to_string();

    let mut packet = Packet::new();
    packet.header.set_type(coap_lite::MessageType::Confirmable);
    packet.header.code = MessageClass::Request(Method::Put); 
    packet.payload = payload.into_bytes(); 
    packet.add_option(CoapOption::UriPath, path.as_bytes().to_vec());

    let message = packet.to_bytes()?;
    socket.send(&message)?;

    let mut buffer = [0; 1024];
    let (size, _) = socket.recv_from(&mut buffer)?;
    let response_packet = Packet::from_bytes(&buffer[..size])?;
    println!("Received response: {:?}", response_packet);

    Ok(())
}
