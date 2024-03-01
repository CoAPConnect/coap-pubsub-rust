use serde_json::json;
use tokio::net::UdpSocket;
use coap_lite::{Packet, RequestType as Method, CoapOption, MessageClass};
use tokio::io::{self, AsyncBufReadExt, BufReader};

async fn update_value(broker_addr: &str, path: &str, key: String, value: i32) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({ key: value }).to_string();

    let mut packet = Packet::new();
    packet.header.set_type(coap_lite::MessageType::Confirmable);
    packet.header.code = MessageClass::Request(Method::Put);
    packet.payload = payload.into_bytes();
    packet.add_option(CoapOption::UriPath, path.as_bytes().to_vec());

    let message = packet.to_bytes()?;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(broker_addr).await?;
    socket.send(&message).await?;

    let mut buffer = [0u8; 1024];
    let (size, _) = socket.recv_from(&mut buffer).await?;
    let response_packet = Packet::from_bytes(&buffer[..size])?;
    println!("Received response: {:?}", response_packet);

    Ok(())
}

#[tokio::main]
async fn main() {
    let broker_addr = "127.0.0.1:5683";
    let mut reader = BufReader::new(io::stdin()).lines();

    loop {
        // Ask user for the topic path
        println!("Insert the topic path: ");
        let topic_path = if let Ok(Some(line)) = reader.next_line().await {
            line.trim().to_string()
        } else {
            println!("Failed to read the topic path.");
            continue;
        };
        let path = format!("{}/data", topic_path);

        // Ask user for the object (key) to update
        println!("Insert object (key) to update: ");
        if let Ok(Some(key)) = reader.next_line().await {
            // Ask user for the value for the object
            println!("Insert value for object '{}': ", key.trim());
            if let Ok(Some(line)) = reader.next_line().await {
                let value: i32 = match line.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Please enter a valid integer.");
                        continue;
                    },
                };

                // Attempt to update the value
                match update_value(broker_addr, &path, key.trim().to_string(), value).await {
                    Ok(_) => println!("{} updated successfully.", key.trim()),
                    Err(e) => println!("Failed to update {}: {}", key.trim(), e),
                }
            }
        }
    }
}
