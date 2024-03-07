use coap::server::{Listener, UdpCoapListener};
use coap_lite::{CoapOption, CoapRequest, RequestType as Method};
use coap::Server;
use resource::DataResource;
use tokio::runtime::Runtime;
use std::net::{SocketAddr, UdpSocket};
mod resource;
use resource::Topic;
use resource::TopicCollection;
use serde_json::json;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

// Topic Collection resource to store all topic-related data
// Lock the mutex to access the topic_collection
// let locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
// Accessing the TopicCollection from the mutex guard
// let topic_collection_ref: &TopicCollection = &*locked_topic_collection;
// Or if mutable collection is needed:
// let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
// let mut topic_collection_ref = Arc::get_mut(&mut locked_topic_collection)

lazy_static! {
    static ref TOPIC_COLLECTION_MUTEX: Mutex<Arc<TopicCollection>> = Mutex::new(Arc::new(TopicCollection::new("TopicCollection".to_string())));
}

fn handle_broker_discovery(req: &mut CoapRequest<SocketAddr>){
    println!("Handling broker discovery");

    let response = req.response.as_mut().unwrap();
    response.message.add_option(CoapOption::ContentFormat, (b"127.0.0.1:5683").to_vec());

    // actual data is sent in the contentformat option, this line is for testing purposes
    response.message.payload = (b"127.0.0.1:5683").to_vec()
}

fn handle_discovery(req: &mut CoapRequest<SocketAddr>) {
    println!("Handling topic discovery");

    // Lock the mutex to access the topic_collection
    let locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    // Accessing the TopicCollection from the mutex guard
    let topic_collection_ref: &TopicCollection = &*locked_topic_collection;

    let topics = topic_collection_ref.get_topics();
    let topic_list: Vec<String> = topics.iter().map(|topic| topic.get_topic_name().to_owned()).collect();

    let payload = json!({"topics": topic_list}).to_string();
    let payload_clone = payload.clone();

    if let Some(ref mut message) = req.response { 
        message.message.payload = payload.into_bytes();
    }
    println!("Topic discovery response sent with payload: {}", payload_clone);
}

fn handle_subscribe(req: &mut CoapRequest<SocketAddr>, topic_name: &str, local_addr: SocketAddr) {
    println!("Beginning subscription handling");
    
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    let topic_collection_ref = Arc::get_mut(&mut locked_topic_collection).unwrap();

    
    // Check if the topic exists
    if let Some(topic) = topic_collection_ref.find_topic_by_name(topic_name) {
        // Topic exists, add subscriber
        let data_path = topic.get_topic_data();
        let data = topic_collection_ref.get_data_from_path_mut(data_path.to_string());
        data.add_subscriber(local_addr);

        println!("Current subscribers for {}: {:?}",topic_name.to_string(), data.get_subscribers());

        // Prepare a success response
        if let Some(ref mut message) = req.response {
            // payload message just for testing purposes
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

    update_topic_data(req, topic_name).await;

// Notify all subscribers of the update
let locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
let topic_collection_ref: &TopicCollection = &*locked_topic_collection;
let topic = topic_collection_ref.find_topic_by_name(topic_name).unwrap();
let topic_data_path = topic.get_topic_data().to_string().clone();

for subscriber in topic_collection_ref.get_data_from_path(topic_data_path).get_subscribers() {
    let path = topic.get_topic_data().to_string().clone();
    let resource_clone = topic_collection_ref.get_data_from_path(path.clone()).get_data().clone();
    
    // Clone the necessary data and move it into the async block
    let subscriber_clone = subscriber.clone();
    let resource_clone = resource_clone.clone();
    tokio::spawn(async move {
        if let Err(e) = inform_subscriber(subscriber_clone, &resource_clone).await {
            eprintln!("Failed to notify subscriber {}: {}", subscriber_clone, e);
        }
    });
}

if let Some(ref mut message) = req.response {
    message.message.payload = b"Resource updated successfully".to_vec();
    println!("{} was updated", topic_name);
} else {
    // Topic not found
    if let Some(ref mut message) = req.response {
        message.message.payload = b"Topic not found".to_vec();
    }
}

}

async fn update_topic_data(req: &mut CoapRequest<SocketAddr>, topic_name: &str) {
    let payload = match String::from_utf8(req.message.payload.clone()) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to decode payload as UTF-8");
            return;
        }
    };

    // Lock the mutex
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();

    // Obtain a mutable reference to the TopicCollection inside the Arc
    if let Some(topic_collection_ref) = Arc::get_mut(&mut locked_topic_collection) {
        // Attempt to find the topic by name
        if let Some(topic) = topic_collection_ref.find_topic_by_name_mut(topic_name) {
            // Action is "data", update the topic's resource
            topic.set_topic_data(payload.clone());
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

/// Initializes 3 topics to topic collection with names "topic1, topic2 & topic3"
/// paths are 123, data/123 ... 456, data/456, ... 789, data/789
fn initialize_topics() {
    // lock mutex
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    // Accessing the TopicCollection from the mutex guard
    let topic_collection = match Arc::get_mut(&mut locked_topic_collection) {
        Some(topic_collection) => topic_collection,
        None => {
            // Handle the case where Arc::get_mut() returns None
            println!("Failed to obtain mutable reference to TopicCollection");
            return; // Or any other appropriate error handling
        }
    };

    let example_data = "{temperature: 20}";
    let data_path1 = "data/123".to_string();
    let data_path2 = "data/456".to_string();
    let data_path3 = "data/789".to_string();

    let mut topic1 = Topic::new("topic1".to_string(), "core.ps.conf".to_string());
    topic1.set_topic_uri("123".to_string());
    topic1.set_topic_data(data_path1.clone());
    let mut topic2 = Topic::new("topic2".to_string(), "core.ps.conf".to_string());
    topic1.set_topic_uri("456".to_string());
    topic2.set_topic_data("data/456".to_string());
    let mut topic3 = Topic::new("topic3".to_string(), "core.ps.conf".to_string());
    topic1.set_topic_uri("456".to_string());
    topic3.set_topic_data("data/789".to_string());


    topic_collection.add_topic(topic1);
    topic_collection.add_topic(topic2);
    topic_collection.add_topic(topic3);
    // Add as many topics / with specific settings as needed for testing

    let mut data1 = DataResource::new(data_path1.clone(), "123".to_string());
    data1.set_data(data_path1.clone());
    topic_collection.set_data(data_path1, data1);

    let mut data2 = DataResource::new(data_path2.clone(), "456".to_string());
    data2.set_data(example_data.to_string());
    topic_collection.set_data(data_path2.clone(), data2);

    let mut data3 = DataResource::new(data_path3.clone(), "789".to_string());
    data3.set_data(example_data.to_string());
    topic_collection.set_data(data_path3.clone(), data3);
}

fn main() {
    initialize_topics();

    let addr = "127.0.0.1:5683";
    Runtime::new().unwrap().block_on(async move {
        let socket_local = tokio::net::UdpSocket::bind(addr).await.unwrap();

        // create server from listeners TODO add multicast address as non-blocking
        let mut listeners: Vec<Box<dyn Listener>> = Vec::new();
        let listener1 = Box::new(UdpCoapListener::from_socket(socket_local));
        listeners.push(listener1);
        let mut server = Server::from_listeners(listeners);

        // remove basic functionality of handling get requests with observe setting
        server.disable_observe_handling(true).await;
        
        println!("Server up on {}, listening for requests", addr);

        // run the server and process requests
        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            match request.get_method() {
                &Method::Get => handle_get(&mut *request),
                &Method::Post => handle_post(&request),
                &Method::Put => handle_put(&mut *request).await,
                &Method::Delete => handle_delete(&mut *request).await,
                _ => println!("Error, request by method that is not supported."),
            };
            // respond to request
            return request;
        }).await.unwrap();
    });
}
