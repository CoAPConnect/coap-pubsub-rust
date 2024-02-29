use coap::server::{Listener, UdpCoapListener};
use coap_lite::{CoapOption, CoapRequest, ContentFormat, RequestType as Method};
use coap::Server;
use tokio::runtime::Runtime;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
// use resource::CoapResource;
use std::collections::HashMap;
use std::ops::Index;
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

fn handle_broker_discovery(req: &mut CoapRequest<SocketAddr>){
    println!("Handling broker discovery");

    let mut response = req.response.as_mut().unwrap();
    response.message.add_option(CoapOption::ContentFormat, (b"127.0.0.1:5683").to_vec());
    response.message.payload = (b"127.0.0.1:5683").to_vec()
}

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
            message.message.payload = b"Subscribed successfully".to_vec();
            println!("{} subscribed to {}", local_addr.to_string(), topic_name);
        }
    } else {
        // Topic does not exist, prepare an error response
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
        }
    }
}

fn handle_invalid_path(req: &CoapRequest<SocketAddr>) {
    // Handle unrecognized paths
    let path = req.get_path();
    println!("Invalid path requested: {}", path);

    let src = req.source.unwrap();
    println!("Requested by: {}", src);
    // Set an appropriate response indicating the error
}

fn handle_get(req: &mut CoapRequest<SocketAddr>) {
    let path = req.get_path(); // Extract the URI path from the request

    // Split the path into components for easier pattern matching
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    match components.as_slice() {
        ["discovery"] => {
            handle_discovery(req);
        },
        [".well-known", "core?rt=core.ps"] => {
            // Handle discovery request
            //handle_discovery(req);
            handle_broker_discovery(req);
        },
        [topic, "subscribe"] => {
            handle_subscribe(req, topic, req.source.unwrap());
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
    let local_addr = req.source.unwrap();

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
            println!("{} updated {}", local_addr.to_string(), topic_name);
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
        [topic_name, "delete"] => {
            delete_topic(req, topic_name, req.source.unwrap());
        },
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}

fn delete_topic(req: &mut CoapRequest<SocketAddr>, topic_name: &str, local_addr: SocketAddr) {
    println!("Deleting topic: {}", topic_name);
    let mut topics = TOPIC_MAP.lock().unwrap(); // Lock the topic map for safe access

    if topics.remove(topic_name).is_some() {
        // Topic found and removed
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic deleted successfully".to_vec();
            println!("{} deleted {}", local_addr.to_string(), topic_name);
        }
    } else {
        // Topic not found
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
}

fn main() {
    initialize_topics();
    let addr = "127.0.0.1:5683";

    Runtime::new().unwrap().block_on(async move {
        let socket_local = tokio::net::UdpSocket::bind(addr).await.unwrap();
        let socket_multi = tokio::net::UdpSocket::bind("0.0.0.0:5683").await.unwrap();

        // listeners on 127.0.0.1:5683 and all coap multicast addresses
        let mut listeners: Vec<Box<dyn Listener>> = Vec::new();
        let listener1 = Box::new(UdpCoapListener::from_socket(socket_local));

        listeners.push(listener1);
        let server = Server::from_listeners(listeners);
        
        println!("Server up on {}, listening to all coap multicasts", addr);

        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            match request.get_method() {
                &Method::Get => handle_get(&mut *request),
                &Method::Post => handle_post(&request),
                &Method::Put => handle_put(&mut *request).await,
                &Method::Delete => handle_delete(&mut *request).await,
                _ => println!("request by other method"),
            };
            return request;
        }).await.unwrap();
    });
}
