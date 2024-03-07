use coap::client::ObserveMessage;
use coap::UdpCoAPClient;
use coap_lite::{
    CoapOption, CoapRequest, CoapResponse, Packet, RequestType as Method
};
use std::io::{self, Write};
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio;
use tokio::net::UdpSocket;
use std::error::Error;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
    static ref LISTENER_SOCKET: Mutex<Option<Arc<UdpSocket>>> = Mutex::new(None);
}

#[tokio::main]
async fn main() {
    handle_command().await;
}

async fn handle_command() {
    let discovery_url = "coap://127.0.0.1:5683/discovery";

    loop {
        println!("Enter command number:");
        println!("1. discovery");
        println!("2. subscribe <TopicName>");
        println!("5. create topic <TopicName> <ResourceType>");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match args.as_slice() {
            ["1"] | ["topic discovery"] => {
                discovery(discovery_url).await;
            },
            ["2", topic_name] | ["subscribe", topic_name] => {
                subscribe(topic_name).await;
            },
            ["5", topic_name, resource_type] | ["create topic", topic_name, resource_type] => { 
                create_topic(topic_name, resource_type).await; 
            },
            _ => println!("Invalid command. Please enter 'discovery' or 'subscribe <TopicName>'."),
        }
    }
}

async fn subscribe(topic_name: &str) -> Result<(), Box<dyn Error>> {

    let listen_socket = {
        let mut ls = LISTENER_SOCKET.lock().unwrap();
        if ls.is_none() {
            let socket = UdpSocket::bind("127.0.0.1:0").await?;
            *ls = Some(Arc::new(socket));
        }
        ls.as_ref().unwrap().clone()
    };

    let local_addr = listen_socket.local_addr()?;

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_method(Method::Get);
    request.set_path(&format!("/subscribe/{}", topic_name));
    request.message.set_observe_value(0);

    let packet = request.message.to_bytes().unwrap();
    listen_socket.send_to(&packet[..], "127.0.0.1:5683").await.expect("Could not send the data");

    let handle = tokio::spawn(async move {
        listen_for_messages(listen_socket).await;
    });

    return Ok(());
}

/// Listen for responses and future publifications on followed topics
/// In the future should check that the response has observe value to see subscription was ok
async fn listen_for_messages(socket: Arc<UdpSocket>) {
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                // Successfully received a message
                let packet = Packet::from_bytes(&buf[..len]).unwrap();
                let request = CoapRequest::from_packet(packet, src);
                let msg = String::from_utf8(request.message.payload).unwrap();
                println!("Received message from {}: {}", src, msg);
            },
            Err(e) => {
                // An error occurred
                eprintln!("Error receiving message: {}", e);
                break;
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

async fn create_topic(topic_name: &str, resource_type: &str) {
    let url = "coap://127.0.0.1:5683/ps"; 
    let payload = json!({"topic-name": topic_name, "resource-type": resource_type}).to_string();
    let payload_bytes = payload.into_bytes();

    match UdpCoAPClient::post(&url, payload_bytes).await {
        Ok(response) => {
            println!("Topic created successfully:");
            if let Some(uri_path) = response.message.get_option(CoapOption::UriPath) {
                println!("Location-Path: {:?}", uri_path);
            } else {
                println!("Location-Path not found in response.");
            }
            println!("Response Payload: {}", String::from_utf8(response.message.payload).unwrap());
        },
        Err(e) => {
            println!("Error creating topic: {}", e);
        }
    }
}
