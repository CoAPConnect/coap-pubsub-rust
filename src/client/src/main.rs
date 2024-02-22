use coap::client::ObserveMessage;
use coap::UdpCoAPClient;
use std::io::{self, Write};
use std::io::ErrorKind;
use tokio;
use tokio::net::UdpSocket;
use std::error::Error;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type ListenerChannel = mpsc::Sender<()>;
type ListenerHandle = tokio::task::JoinHandle<()>;
// Store the listener handle, stop channel, and local address
type ListenerInfo = (ListenerHandle, ListenerChannel, String);
type ListenerMap = Arc<Mutex<HashMap<String, ListenerInfo>>>;


#[tokio::main]
async fn main() {
    handle_command().await;
}

async fn handle_command() {
    let listeners: ListenerMap = Arc::new(Mutex::new(HashMap::new()));
    let discovery_url = "coap://127.0.0.1:5683/discovery";

    loop {
        println!("Enter command:");
        println!("1. discovery");
        println!("2. subscribe <TopicName>");
        println!("3. unsubscribe <TopicName>");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match args.as_slice() {
            ["discovery"] => {
                discovery(discovery_url).await;
            },
            ["subscribe", topic_name] => {
                let listeners_clone = listeners.clone();
                subscribe(topic_name, listeners_clone).await;
            },
            ["unsubscribe", topic_name] => {
                let listeners_clone = listeners.clone();
                unsubscribe(topic_name, listeners_clone).await;
            },
            _ => println!("Invalid command. Please enter 'discovery' or 'subscribe <TopicName>'."),
        }
    }
}

async fn subscribe(topic_name: &str, listeners: ListenerMap) -> Result<(), Box<dyn Error>> {
   let (tx, rx) = mpsc::channel(1);
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
   let handle = tokio::spawn(async move {
       listen_for_messages(socket, rx).await;
   });

   listeners.lock().unwrap().insert(topic_name.to_string(), (handle, tx, local_addr.to_string()));

   return Ok(());
}

async fn unsubscribe(topic_name: &str, listeners: ListenerMap) -> Result<(), Box<dyn Error>> {
    // Attempt to remove the listener information using the topic name
    if let Some((handle, tx, local_addr_str)) = listeners.lock().unwrap().remove(topic_name) {
        // Send the stop signal to the listener task
        tx.send(()).await.unwrap();
        // Wait for the listener task to finish
        handle.await.unwrap();

        // The address of the CoAP broker
        let broker_addr = "127.0.0.1:5683";

        // Construct the unsubscribe URL using the local address of the listener
        let unsubscribe_url = format!("coap://{}/unsubscribe/{}/{}", broker_addr, topic_name, local_addr_str);

        // Send the unsubscribe request
        match UdpCoAPClient::delete(&unsubscribe_url).await {
            Ok(response) => {
                println!("Unsubscribed successfully: {}", String::from_utf8_lossy(&response.message.payload));
            },
            Err(e) => {
                eprintln!("Error unsubscribing: {:?}", e);
            }
        }
    } else {
        println!("Listener for topic '{}' not found.", topic_name);
    }

    Ok(())
}

async fn listen_for_messages(socket: UdpSocket, mut stop_signal: mpsc::Receiver<()>) {
    let mut buf = [0u8; 1024];
    loop {
        tokio::select! {
            _ = stop_signal.recv() => {
                println!("Stop listening for topic");
                break;
            }
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, src)) => {
                        println!("Received message from {}: {}", src, String::from_utf8_lossy(&buf[..len]));
                    },
                    Err(e) => {
                        eprintln!("Error receiving message: {}", e);
                        break;
                    }
                }
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
