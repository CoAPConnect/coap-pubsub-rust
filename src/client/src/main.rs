use coap::client::ObserveMessage;
use coap::UdpCoAPClient;
use coap_lite::{
    CoapOption, CoapRequest, RequestType as Method
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
        println!("3. unsubscribe <TopicName>");
        println!("4. multicast broker discovery");

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
            ["3", topic_name] | ["unsubscribe", topic_name] => {
                unsubscribe(topic_name).await;
            },
            ["4"] | ["multicast discovery"] => {
                multicast_discovery().await;
            },
            _ => println!("Invalid command. Please enter 'discovery' or 'subscribe <TopicName>'."),
        }
    }
}

async fn multicast_discovery(){
    let addr = "127.0.0.1:5683";
    println!("Multicast attempt start");

    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps");

    //segment is ipv6 segment for multicast, need to be called on all segments we want to use, but in our case ipv4 is used so "0" is enough for now
    let segment: u8 = 0;
    UdpCoAPClient::send_all_coap(&client, &request, segment).await.unwrap();
    let response = client.receive_raw_response().await.unwrap();
    let pay = std::str::from_utf8((response.message.get_first_option(CoapOption::ContentFormat)).unwrap());
    match pay {
        Ok(pay) => {
            println!("Payload: {}", pay);
        }
        Err(err) => {
            println!("Error converting payload to string: {}", err);
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

    let packet = request.message.to_bytes().unwrap();
    listen_socket.send_to(&packet[..], "127.0.0.1:5683").await.expect("Could not send the data");

    let handle = tokio::spawn(async move {
        listen_for_messages(listen_socket).await;
    });

    return Ok(());
}

async fn listen_for_messages(socket: Arc<UdpSocket>) {
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                // Successfully received a message
                println!("Received message from {}: {}", src, String::from_utf8_lossy(&buf[..len]));
            },
            Err(e) => {
                // An error occurred
                eprintln!("Error receiving message: {}", e);
                break;
            }
        }
    }
}


async fn unsubscribe(topic_name: &str) -> Result<(), Box<dyn Error>> {

    let listen_socket = {
        let mut ls = LISTENER_SOCKET.lock().unwrap();
        if ls.is_none() {
            let socket = UdpSocket::bind("127.0.0.1:0").await?;
            *ls = Some(Arc::new(socket));
        }
        ls.as_ref().unwrap().clone()
    };

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_method(Method::Delete); // Set method to DELETE
    request.set_path(&format!("/unsubscribe/{}", topic_name));

    // Convert the CoAP request to bytes and send it
    let packet = request.message.to_bytes().unwrap();
    listen_socket.send_to(&packet[..], "127.0.0.1:5683").await.expect("Could not send the data");

    Ok(())
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
