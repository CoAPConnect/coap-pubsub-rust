use coap::client::ObserveMessage;
use coap::UdpCoAPClient;
use coap_lite::{
    CoapOption, CoapRequest, CoapResponse, Packet, RequestType as Method
};
use std::io::{self, Write};
use std::error::Error;
use std::io::{ErrorKind, Error as IoError};
use std::net::SocketAddr;
use tokio;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::convert::Into;
use lazy_static::lazy_static;
use serde_json::json;

lazy_static! {
    static ref LISTENER_SOCKET: Mutex<Option<Arc<UdpSocket>>> = Mutex::new(None);
}

#[tokio::main]
async fn main() {
    handle_command().await;
}
static GLOBAL_URL: &str = "127.0.0.1:5683";

async fn handle_command() {
    let discovery_url = "coap://".to_owned()+GLOBAL_URL+"/discovery";

    loop {
        println!("Enter command number:");
        println!("1. topic discovery");
        println!("2. subscribe <TopicName>");
        println!("3. create topic <TopicName>");
        println!("5. publish or update dataresource: PUT <DataURI> <Payload>");
        println!("6. delete topic configuration: DELETE <TopicURI>");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match args.as_slice() {
            ["1"] | ["topic discovery"] => {
                discovery(&discovery_url).await;
            },
            ["2", topic_name] | ["subscribe", topic_name] => {
                subscribe(topic_name).await;
            },
            ["3", topic_name] | ["create topic", topic_name]=>{
                create_topic(topic_name).await;
            },
            ["5", topic_name, payload] | ["PUT", topic_name, payload] => {
                update_topic(topic_name, payload).await;
            },
            ["6", topic_name] | ["DELETE", topic_name] => {
                delete_topic(topic_name).await;
            },
            _ => println!("Invalid command. Please enter 'discovery' or 'subscribe <TopicName>'."),
        }
    }
}

/// Function that handles formatting the server reply in the case a response comes through. Prints the response code and the payload.
fn server_reply(response: CoapResponse){
    let code = response.message.header.code;
            println!(
                "Server reply: {} {}",
                code.to_string(),
                String::from_utf8(response.message.payload).unwrap()
            );
}
fn server_error(e: &IoError) {
    match e.kind() {
        ErrorKind::WouldBlock => println!("Request timeout"), // Unix
        ErrorKind::TimedOut => println!("Request timeout"),   // Windows
        _ => println!("Request error: {:?}", e),
    }
}


async fn delete_topic(topic_name: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/{}", "coap://".to_owned()+GLOBAL_URL,topic_name);
    println!("Client request: {}", url);

    match UdpCoAPClient::delete(&url).await {
        Ok(response) => {
            server_reply(response);
            Ok(())
        }
        Err(e) => {
            server_error(&e);
            Err(Box::new(e))
        }
    }
}

async fn update_topic(topic_name: &str, payload: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/{}/data","coap://".to_owned()+GLOBAL_URL, topic_name);
    let data = payload.as_bytes().to_vec();
    println!("Client request: {}", url);

    match UdpCoAPClient::put(&url, data).await {
        Ok(response) => {
            server_reply(response);
            Ok(())
        }
        Err(e) => {
            server_error(&e);
            Err(Box::new(e))
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

    //let local_addr = listen_socket.local_addr()?;

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_method(Method::Get);
    request.set_path(&format!("/{}/subscribe", topic_name));
    request.message.set_observe_value(0);

    let packet = request.message.to_bytes().unwrap();
    listen_socket.send_to(&packet[..], &GLOBAL_URL).await.expect("Could not send the data");

    let _handle = tokio::spawn(async move {
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

async fn create_topic(topic_name: &str) {
    let url = "coap://".to_owned()+GLOBAL_URL+"/ps"; 
    let resource_type="core.ps.conf";
    let payload = json!({"topic-name": topic_name, "resource-type": resource_type}).to_string();
    let payload_bytes = payload.into_bytes();
    

    match UdpCoAPClient::post(&url, payload_bytes).await {
        Ok(response) => {
            let payload_string = String::from_utf8(response.message.payload).unwrap();
            let code = response.message.header.get_code().to_string();
            println!("Server reply: {} {}",code,payload_string);
        },
        Err(e) => {
            println!("Error creating topic: {}", e);
        }
    }
}
