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

    let response = req.response.as_mut().unwrap();
    response.message.add_option(CoapOption::ContentFormat, (b"127.0.0.1:5683").to_vec());

    // actual data is sent in the conrentformat option, this line is for testing purposes
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
    println!("Beginning subscription handling");
    
    let mut topics = TOPIC_MAP.lock().unwrap(); // Lock the topic map for safe access

    // Check if the topic exists
    if let Some(topic) = topics.get_mut(topic_name) {
        // Topic exists, add subscriber

        let subscriber = Subscriber { addr: local_addr};
        topic.subscribers.push(subscriber);

        // Prepare a success response
        if let Some(ref mut message) = req.response {
            message.message.payload = format!("Subscribed to {}", topic_name).into_bytes();
            message.message.set_content_format(coap_lite::ContentFormat::try_from(110).unwrap());
            message.message.set_observe_value(10001);
        }
        println!("{} subscribed to {}", local_addr.to_string(), topic_name);
    } else {
        // Topic does not exist, prepare an error response
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
            message.set_status(coap_lite::ResponseType::NotFound);
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
        [".well-known", "core"] | [".well-known", "core?rt=core.ps"] => {
            handle_broker_discovery(req);
        },
        ["subscribe", topic] => {
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
            println!("{} was updated", topic_name);
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

/// Creates a new topic
fn create_topic(topic_name: &String, resource_type: &String, req: &mut coap_lite::CoapRequest<SocketAddr>) {
    let topic = Topic::new();
    let mut topic_map = TOPIC_MAP.lock().unwrap();
    let topic_name_cloned = topic_name.clone();
    topic_map.insert(topic_name_cloned, topic);
    println!("Topic '{}' of type '{}' added to the topic map.", topic_name, resource_type);

    if let Some(ref mut message) = req.response {
        message.message.payload = b"Topic created succesfully".to_vec();
        message.set_status(coap_lite::ResponseType::Created);
    }
}

/// Handles post requests, i.e. topic creation and topic configuration updates
fn handle_post(req:&mut Box<CoapRequest<SocketAddr>>){
    // handle topic config etc
     // Extract payload from request
     let payload = String::from_utf8_lossy(&req.message.payload);

     // Parse payload to obtain topic-name and resource-type
     let parsed_payload: serde_json::Value = serde_json::from_str(payload.as_ref()).unwrap();
     let topic_name: &String = &parsed_payload["topic-name"].as_str().unwrap().to_string();
     let resource_type: &String = &parsed_payload["resource-type"].as_str().unwrap().to_string();

    // Add the topic to the topic map
    create_topic(topic_name, resource_type, req);
}

async fn handle_delete(req: &mut CoapRequest<SocketAddr>) {
    let path = req.get_path(); // Extract the URI path from the request
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    match components.as_slice() {
        // Add resource deletion command here
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}

fn handle_resource_deletion_or_invalid_path(req: &mut CoapRequest<SocketAddr>, components: &[&str]) {
    // Implement resource deletion or handle invalid path
    // This function is a placeholder for actual logic
    println!("Resource deletion or invalid path handling is not implemented.");
}

/// This function initializes the necessary topics for testing
/// Later will change to initialise the correct uri paths too as specified in rfc6690 (wellknown/core and ps)
fn initialize_topics() {
    let mut topics = TOPIC_MAP.lock().unwrap();
    // TODO:
    // this has to change later on to work via revamped resources
    // Add some predefined topics for testing purposes
    topics.insert("topic1".to_string(), Topic::new());
    topics.insert("topic2".to_string(), Topic::new());
    topics.insert("topic3".to_string(), Topic::new());
    // Add as many topics as needed for testing
}

fn main() {
    initialize_topics();

    let addr = "127.0.0.1:5683";
    Runtime::new().unwrap().block_on(async move {
        let socket_local = tokio::net::UdpSocket::bind(addr).await.unwrap();

        let mut listeners: Vec<Box<dyn Listener>> = Vec::new();
        let listener1 = Box::new(UdpCoapListener::from_socket(socket_local));

        listeners.push(listener1);

        let mut server = Server::from_listeners(listeners);

        server.disable_observe_handling(true).await;
        
        println!("Server up on {}, listening to coap multicasts", addr);

        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            match request.get_method() {
                &Method::Get => handle_get(&mut *request),
                &Method::Post => handle_post(&mut request),
                &Method::Put => handle_put(&mut *request).await,
                &Method::Delete => handle_delete(&mut *request).await,
                _ => println!("request by other method"),
            };



            // respond to request
            return request;
        }).await.unwrap();
    });
}
