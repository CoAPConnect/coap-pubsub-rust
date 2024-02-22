use coap_lite::{RequestType as Method, CoapRequest};
use coap::Server;
use tokio::runtime::Runtime;
use std::net::{SocketAddr, UdpSocket};
// use resource::CoapResource;
use std::collections::HashMap;
use serde_json::json;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

struct Subscriber {
    addr: SocketAddr,
}

struct Topic {
    subscribers: Vec<Subscriber>,
    resource: String,
}

impl Topic {
    fn new() -> Self {
        Topic {
            subscribers: Vec::new(),
            resource: String::new(),
        }
    }
}

static TOPIC_MAP: Lazy<Arc<Mutex<HashMap<String, Topic>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

fn handle_discovery(req: &mut CoapRequest<SocketAddr>) {
    println!("Handling discovery");

    let topics = TOPIC_MAP.lock().unwrap();
    let topic_list: Vec<String> = topics.keys().cloned().collect();
    let payload = json!({"topics": topic_list}).to_string();

    if let Some(ref mut message) = req.response { 
        message.message.payload = payload.into_bytes();
    }
    println!("Discovery response sent");
}

fn handle_subscribe(req: &mut CoapRequest<SocketAddr>, topic_name: &str, local_addr: SocketAddr) {
    println!("Subscribing to topic: {}", topic_name);

    let mut topics = TOPIC_MAP.lock().unwrap(); // Lock the topic map for safe access

    // Check if the topic exists
    if let Some(topic) = topics.get_mut(topic_name) {
        // Topic exists, add subscriber

        let subscriber = Subscriber { addr: local_addr};
        topic.subscribers.push(subscriber);

        // Prepare a success response
        if let Some(ref mut message) = req.response {
            message.message.payload = format!("Subscribed to {}", topic_name).into_bytes();
        }
        println!("{} subscribed to {}", local_addr.to_string(), topic_name);
    } else {
        // Topic does not exist, prepare an error response
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
        }
    }
}


fn handle_invalid_path(req: &CoapRequest<SocketAddr>) {
    // Handle unrecognized paths
    println!("Invalid path requested");
    // Set an appropriate response indicating the error
}

fn handle_get(req: &mut CoapRequest<SocketAddr>) {
    let path = req.get_path(); // Extract the URI path from the request

    // Split the path into components for easier pattern matching
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    match components.as_slice() {
        ["discovery"] => {
            // Handle discovery request
            handle_discovery(req);
        },
        ["subscribe", topic, address] => {
            // Handle subscription request
            if let Ok(address) = address.parse::<SocketAddr>() {
                handle_subscribe(req, topic, address);
            } else {
                eprintln!("Invalid address format: {}", address);
            }
        },
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}

async fn handle_put(req: &mut CoapRequest<SocketAddr>) {
    let path_str = req.get_path();
    let components: Vec<&str> = path_str.split('/').filter(|s| !s.is_empty()).collect();

    // Now expecting at least 2 components: "topicName" and "data"
    if components.len() < 2 {
        eprintln!("Invalid path format. Received: {}", path_str);
        return;
    }

    let topic_name = components[0]; // Adjusted index
    let action = components[1]; // Adjusted index

    // Ensure the action is what we expect, e.g., "data"
    if action != "data" {
        eprintln!("Unsupported action: {}", action);
        return;
    }

    let payload = match String::from_utf8(req.message.payload.clone()) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to decode payload as UTF-8");
            return;
        }
    };

    let mut topics = TOPIC_MAP.lock().unwrap();

    if let Some(topic) = topics.get_mut(topic_name) {
        // Action is "data", update the topic's resource
        topic.resource = payload.clone();

        // Notify all subscribers of the update
        for subscriber in &topic.subscribers {
            let resource_clone = topic.resource.clone();
            let subscriber_addr = subscriber.addr;
            
            tokio::spawn(async move {
                if let Err(e) = inform_subscriber(subscriber_addr, &resource_clone).await {
                    eprintln!("Failed to notify subscriber {}: {}", subscriber_addr, e);
                }
            });
        }

        if let Some(ref mut message) = req.response {
            message.message.payload = b"Resource updated successfully".to_vec();
        }
    } else {
        // Topic not found
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
        }
    }
}


async fn inform_subscriber(addr: SocketAddr, resource: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize your resource as JSON, or use it directly if it's already a JSON string
    let payload = resource.as_bytes();

    // Placeholder for asynchronous network call
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&payload, &addr)?;

    Ok(())
}


fn handle_post(req:&Box<CoapRequest<SocketAddr>>){
    // handle topic config etc
}

async fn handle_delete(req: &mut CoapRequest<SocketAddr>) {
    let path = req.get_path(); // Extract the URI path from the request
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    match components.as_slice() {
        ["unsubscribe", topic_name, subscriber_addr_str] => {
            if let Ok(subscriber_addr) = subscriber_addr_str.parse::<SocketAddr>() {
                unsubscribe_topic(req, topic_name, subscriber_addr);
            } else {
                // Handle invalid subscriber address
                if let Some(ref mut message) = req.response {
                    message.message.payload = b"Invalid subscriber address".to_vec();
                }
            }
        },
        // Add resource deletion command here
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}

fn unsubscribe_topic(req: &mut CoapRequest<SocketAddr>, topic_name: &str, subscriber_addr: SocketAddr) {
    let mut topics = TOPIC_MAP.lock().unwrap(); // Lock the topic map for safe access

    if let Some(topic) = topics.get_mut(topic_name) {
        // Topic found, attempt to remove subscriber
        if let Some(index) = topic.subscribers.iter().position(|s| s.addr == subscriber_addr) {
            // Subscriber found, remove it
            topic.subscribers.remove(index);
            println!("{} unsubscribed from {}", subscriber_addr, topic_name);

            // Prepare a success response
            if let Some(ref mut message) = req.response {
                message.message.payload = format!("Unsubscribed from {}", topic_name).into_bytes();
            }
        } else {
            // Subscriber not found, prepare an error response
            if let Some(ref mut message) = req.response {
                message.message.payload = b"Subscriber not found".to_vec();
            }
        }
    } else {
        // Topic not found, prepare an error response
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
        }
    }
}

fn handle_resource_deletion_or_invalid_path(req: &mut CoapRequest<SocketAddr>, components: &[&str]) {
    // Implement resource deletion or handle invalid path
    // This function is a placeholder for actual logic
    println!("Resource deletion or invalid path handling is not implemented.");
}

fn initialize_topics() {
    let mut topics = TOPIC_MAP.lock().unwrap();
    // Add some predefined topics
    topics.insert("topic1".to_string(), Topic::new());
    topics.insert("topic2".to_string(), Topic::new());
    topics.insert("topic3".to_string(), Topic::new());
    // Add as many topics as needed for testing
}

fn main() {
    initialize_topics();

    let addr = "127.0.0.1:5683";
    Runtime::new().unwrap().block_on(async move {
        let server = Server::new_udp(addr).unwrap();
        println!("Server up on {}", addr);

        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            match request.get_method() {
                &Method::Get => handle_get(&mut *request),
                &Method::Post => handle_post(&request),
                &Method::Put => handle_put(&mut *request).await,
                &Method::Delete => handle_delete(&mut *request).await,
                _ => println!("request by other method"),
            };



            // respond to request
            return request;
        }).await.unwrap();
    });
}
