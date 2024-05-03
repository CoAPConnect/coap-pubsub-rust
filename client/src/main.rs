use coap::UdpCoAPClient;
use coap_lite::{CoapRequest, CoapResponse, Packet, RequestType as Method};
use std::io::{self, Write};
use std::error::Error;
use std::io::{ErrorKind, Error as IoError};
use std::net::SocketAddr;
use tokio;
use tokio::net::UdpSocket;
use std::sync::{Arc, Mutex};
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
static GLOBAL_URL: &str = "127.0.0.1:5681";

async fn handle_command() {
    let discovery_url = "coap://".to_owned()+GLOBAL_URL+"/discovery";

    loop {
        println!("");
        println!("Enter command number:");
        println!("1. topic name/uri/datauri discovery");
        println!("2. subscribe <Topic_data_URI>");
        println!("3. unsubscribe <Topic_data_URI>");
        println!("4. create topic <TopicName>");
        println!("5. update topic data: PUT <Topic_data_URI> <Payload>");
        println!("6. delete topic configuration: DELETE <TopicURI>");
        println!("7. multicast broker discovery");
        println!("8. broker discovery");
        println!("9. read latest data <Topic_data_URI>");
        println!("10. topic-configuration discovery");
        println!("11. topic-data discovery");
        println!("12. topic collection discovery");
        println!("");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let args: Vec<&str> = input.trim().split_whitespace().collect();

        match args.as_slice() {
            ["1"] | ["topic name discovery"] => {
                discovery(&discovery_url).await;
            },
            ["2", topic_data_uri] | ["subscribe", topic_data_uri] => {
                let _ = subscription(topic_data_uri, 0).await;
            },
            ["3", topic_data_uri] | ["unsubscribe", topic_data_uri] => {
                let _ = subscription(topic_data_uri, 1).await;
            },
            ["4", topic_name] | ["create topic", topic_name]=>{
                create_topic(topic_name).await;
            },
            ["5", topic_data_uri, payload] | ["PUT", topic_data_uri, payload] => {
                let _ = update_topic(topic_data_uri, payload).await;
            },
            ["6", topic_name] | ["DELETE", topic_name] => {
                let _ = delete_topic(topic_name).await;
            },
            ["7"] | ["multicast", "broker", "discovery", "uri", "query"] => {
                multicast_discovery_uri_query().await;
            },
            ["8"] | ["broker", "discovery", "uri", "query"] => {
                broker_discovery_uri_query().await;
            },
            ["9", topic_data] | ["read", "latest", "data", topic_data] => {
                let _ = read_latest_topic_data(topic_data).await;
            },
            ["10"] | ["topic", "configuration", "discovery"] => {
                let _ = topic_configuration_discovery().await;
            },
            ["11"] | ["topic", "data", "discovery"] => {
                let _ = topic_data_discovery().await;
            },
            ["12"] | ["topic", "collection", "discovery"] => {
                let _ = topic_collection_discovery().await;
            }
            _ => println!("Invalid command. Please enter one from the list of commands."),
        }
    }
}

/// Performs simple GET request with resource type = core.ps.coll and prints out the response
async fn topic_collection_discovery() {
    println!("Topic collection discovery start");
    let addr = GLOBAL_URL;
    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();
    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps.coll");

    let response = UdpCoAPClient::perform_request(&mut client, request).await.unwrap();
    let pay = String::from_utf8(response.message.payload);
    match pay {
        Ok(pay) => {
            println!("Response: {}", pay);
        }
        Err(err) => {
            println!("Error converting payload to string: {}", err);
        }
    }
}

/// Performs simple GET request with resource type = core.ps.data and prints out the response.
async fn topic_data_discovery() {
    println!("Topic data discovery start");
    let addr = GLOBAL_URL;
    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();
    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps.data");

    let response = UdpCoAPClient::perform_request(&mut client, request).await.unwrap();
    let pay = String::from_utf8(response.message.payload);
    match pay {
        Ok(pay) => {
            println!("Response: {}", pay);
        }
        Err(err) => {
            println!("Error converting payload to string: {}", err);
        }
    }
}

/// Multicast discovery using ipv4 port 5683 and ipv6 segment 0.
async fn multicast_discovery_uri_query(){
    let addr = "0.0.0.0:5683";
    println!("Multicast attempt start with uri query, listening for responses for 1s");

    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps");

    //segment is ipv6 segment for multicast, need to be called on all segments we want to use, but in our case ipv4 is used so "0" is enough for now
    let segment: u8 = 0;
    UdpCoAPClient::send_all_coap(&client, &request, segment).await.unwrap();

    // listens for responses from multiple brokers for 1 second and then times out.
    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < 1 {
        let response = match client.receive_raw_response().await {
            Ok(response) => response,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::TimedOut {
                    println!("No more responses received in 1s");
                    break; // Exit the loop on timeout
                }
                println!("Error receiving response: {}", err);
                break; // Exit the loop on error
            }
        };
        
        let pay = match String::from_utf8(response.message.payload) {
            Ok(pay) => pay,
            Err(err) => {
                println!("Error converting payload to string: {}", err);
                continue; // Skip to the next iteration on error
            }
        };
        println!("Response: {}", pay);
    }
}

/// Broker discovery using known broker address and uri query to define resource-type
async fn broker_discovery_uri_query(){
    let addr = GLOBAL_URL;
    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();
    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps");

    let response = UdpCoAPClient::perform_request(&mut client, request).await.unwrap();
    let pay = String::from_utf8(response.message.payload);
    match pay {
        Ok(pay) => {
            println!("Response: {}", pay);
        }
        Err(err) => {
            println!("Error converting payload to string: {}", err);
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
/// Function that handles formatting the server error in the case an error occurs. Prints the error message.
fn server_error(e: &IoError) {
    match e.kind() {
        ErrorKind::WouldBlock => println!("Request timeout"), // Unix
        ErrorKind::TimedOut => println!("Request timeout"),   // Windows
        _ => println!("Request error: {:?}", e),
    }
}

/// Function that handles deleting a topic configuration. Sends a DELETE request to the server.
async fn delete_topic(topic_uri: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/{}", "coap://".to_owned()+GLOBAL_URL,topic_uri);
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
/// Function that handles updating a topic. Sends a PUT request to the server.
async fn update_topic(topic_data_uri: &str, payload: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("{}/ps/data/{}","coap://".to_owned()+GLOBAL_URL, topic_data_uri);
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
/// Function that handles subscribing to a topic.
/// Sends a GET request to the server with the observe value set to 0 subscription and 1 for unsubscription.
async fn subscription(topic_data_uri: &str, observe_value: u32) -> Result<(), Box<dyn Error>> {

    let listen_socket = {
        let mut ls = LISTENER_SOCKET.lock().unwrap();
        if ls.is_none() {
            let socket = UdpSocket::bind("127.0.0.1:0").await?;
            *ls = Some(Arc::new(socket));
        }
        ls.as_ref().unwrap().clone()
    };

    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_method(Method::Get);

    // Set the path to subscribe or unsubscribe based on the `observe_value` parameter
    let path = format!("ps/data/{}", topic_data_uri);

    request.set_path(&path);
    request.message.set_observe_value(observe_value);

    let packet = request.message.to_bytes().unwrap();
    listen_socket.send_to(&packet[..], &GLOBAL_URL).await.expect("Could not send the data");

    // starts listening to topic, terminates if response doesn't have a observe value set
    let _handle = tokio::spawn(async move {
        listen_for_messages(listen_socket).await;
    });


    return Ok(());
}


/// Listen for responses and future publifications on followed topics
async fn listen_for_messages(socket: Arc<UdpSocket>) {
    let mut buf = [0u8; 1024];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, src)) => {
                // Successfully received a message
                let packet = Packet::from_bytes(&buf[..len]).unwrap();
                let request = CoapRequest::from_packet(packet, src);
                let clone = request.clone();
                let msg = String::from_utf8(clone.message.payload).unwrap();
                println!("Received message from {}: {}", src, msg);

                if let Some(result) = request.message.get_observe_value() {
                    match result {
                        Ok(value) => {
                            // Handle value when it's 1
                            if value == 1 {
                                println!("Stopped listening for topic succesfully");
                                break;
                            } else {
                                // Continue to listen, value is something else than 1.
                                continue;
                            }
                        }
                        Err(err) => {
                            // Handle error when parsing the value
                            println!("Error parsing the observe value, stopping listening: {:?}", err);
                            break;
                        }
                    }
                } else {
                    // Observe value is not present, this is fine on some messages but not on the ones listened on this function
                    eprintln!("Message doesn't have observe value set so it's erroneous, stopping listening");
                    break;
                }
            },
            Err(e) => {
                // An error occurred
                eprintln!("Error receiving message, stopping listening with error message: {}", e);
                break;
            }
        }
    }
}

/// Function that handles the discovery of topics.
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
/// Function that handles the creation of a topic. Name of the topic is mandatory parameter. 
/// Sends a POST request to the server.
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
    
/// Read latest topic data. Sends a GET request to the server.
async fn read_latest_topic_data(topic_data: &str) -> Result<(), Box<dyn Error>> {
    let topic_data_uri = format!("/ps/data/{}", topic_data);
    let url = format!("coap://{}{}", GLOBAL_URL, topic_data_uri);
    println!("Client request: {}", url);

    // Make a GET request to retrieve the latest topic data
    match UdpCoAPClient::get(&url).await {
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
/// Topic configuration discovery.
async fn topic_configuration_discovery() {
    println!("Topic configuration discovery start");
    let addr = GLOBAL_URL;
    let mut client: UdpCoAPClient = UdpCoAPClient::new_udp(addr).await.unwrap();
    let mut request: CoapRequest<SocketAddr> = CoapRequest::new();
    request.set_path(".well-known/core?rt=core.ps.conf");

    let response = UdpCoAPClient::perform_request(&mut client, request).await.unwrap();
    let pay = String::from_utf8(response.message.payload);
    match pay {
        Ok(pay) => {
            println!("Response: {}", pay);
        }
        Err(err) => {
            println!("Error converting payload to string: {}", err);
        }
    }    
}
