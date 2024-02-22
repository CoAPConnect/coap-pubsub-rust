use coap::client::ObserveMessage;
use coap::UdpCoAPClient;
use std::io::{self, Write};
use std::io::ErrorKind;
use tokio;
use tokio::net::UdpSocket;
use std::error::Error;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    handle_command().await;
}

async fn handle_command() {
    let discovery_url = "coap://127.0.0.1:5683/discovery";

    loop {
        println!("Enter command:");
        println!("1. discovery");
        println!("2. subscribe <TopicName>");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match args.as_slice() {
            ["discovery"] => {
                discovery(discovery_url).await;
            },
            ["subscribe", topic_name] => {
                subscribe(topic_name).await;
            },
            _ => println!("Invalid command. Please enter 'discovery' or 'subscribe <TopicName>'."),
        }
    }
}

async fn subscribe(topic_name: &str) -> Result<(), Box<dyn Error>> {
   // Bind a UDP socket to an ephemeral port
   let socket = UdpSocket::bind("127.0.0.1:0").await?;
   let local_addr = socket.local_addr()?;

   // The address of the CoAP broker
   let broker_addr = "127.0.0.1:5683";

   // Construct the subscription URL
   let subscribe_url = format!("coap://{}/subscribe/{}/{}", broker_addr, topic_name, local_addr.to_string());

   match UdpCoAPClient::get(&subscribe_url).await {
    Ok(response) => {
        println!(
            "Server reply: {}",
            String::from_utf8(response.message.payload).unwrap()
        );
    }
    Err(e) => {
        match e.kind() {
            ErrorKind::WouldBlock => println!("Request timeout"), // Unix
            ErrorKind::TimedOut => println!("Request timeout"),   // Windows
            _ => println!("Request error: {:?}", e),
        }
      }
    }
   
   // Spawn a background task to listen for messages
   tokio::spawn(async move {
       listen_for_messages(socket).await;
   });

   return Ok(());
}

async fn listen_for_messages(socket: UdpSocket) {
    let mut buf = [0u8; 1024]; // Buffer for incoming data
    loop {
        match timeout(Duration::from_secs(3600), socket.recv_from(&mut buf)).await {
            Ok(Ok((len, src))) => {
                println!("Received message from {}: {}", src, String::from_utf8_lossy(&buf[..len]));
                // Handle the message as needed
            },
            Ok(Err(e)) => {
                eprintln!("Error receiving message: {}", e);
                break; // Exit the loop if there's an error
            },
            Err(_) => {
                println!("Listening timed out after 3600 seconds");
                break; // Exit the loop after the timeout
            }
        }
    }
}

async fn discovery(url: &str) {

    println!("Client request: {}", url);

    match UdpCoAPClient::get(url).await {
        Ok(response) => {
            println!(
                "Server reply: {}",
                String::from_utf8(response.message.payload).unwrap()
            );
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::WouldBlock => println!("Request timeout"), // Unix
                ErrorKind::TimedOut => println!("Request timeout"),   // Windows
                _ => println!("Request error: {:?}", e),
            }
        }
    }
}
