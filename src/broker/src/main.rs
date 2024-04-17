use coap::server::{Listener, UdpCoapListener};
use coap_lite::link_format::LinkFormatWrite;
use coap_lite::CoapResponse;
use coap_lite::{CoapRequest, ResponseType, RequestType as Method};
//use coap_lite::Message;
use coap::Server;
use resource::DataResource;
use socket2::{Domain, Socket, Type};
use tokio::runtime::Runtime;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
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
enum SubscriptionAction {
    Subscribe,
    Unsubscribe,
}

/// Notifies client of status of request
fn notify_client(response_type: coap_lite::ResponseType, message: &mut coap_lite::CoapResponse, payload: &str){
    message.message.payload = payload.as_bytes().to_vec();
    message.set_status(response_type);
}

/// Handles broker discovery of core.ps, returns ip address of broker
fn handle_broker_discovery(req: &mut CoapRequest<SocketAddr>){
    println!("Handling broker discovery");

    println!("Received request with payload: {}", String::from_utf8(req.message.payload.clone()).unwrap());
    // Set correct responsetypes and content formats in the response
    let response = req.response.as_mut().unwrap();
    response.set_status(ResponseType::Content);
    response.message.set_content_format(coap_lite::ContentFormat::ApplicationLinkFormat);

    // Create the linkformatted response containing the brokers address with rt=core.ps
    let mut buffer = String::new();
    let mut write = LinkFormatWrite::new(&mut buffer);
    write.link("127.0.0.1:5683")
    .attr(coap_lite::link_format::LINK_ATTR_RESOURCE_TYPE, "core.ps");

    println!("Sending response: {}", buffer);
  
    // Return linkformatted response in bytes
    response.message.payload = buffer.as_bytes().to_vec();
}

/// Topic name discovery - not an actual coap pubsub draft method
fn handle_discovery(req: &mut CoapRequest<SocketAddr>) {
    println!("Handling topic name discovery");

    // Lock the mutex to access the topic_collection
    let locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    // Accessing the TopicCollection from the mutex guard
    let topic_collection_ref: &TopicCollection = &*locked_topic_collection;

    let topics = topic_collection_ref.get_topics();
    let topic_list: Vec<String> = topics.values().map(|topic| topic.get_topic_name().to_owned()).collect();

    let payload = json!({"topics": topic_list}).to_string();
    let payload_clone = payload.clone();

    if let Some(ref mut message) = req.response { 
        message.message.payload = payload.into_bytes();
    }
    println!("Topic name discovery response sent with payload: {}", payload_clone);
}

/// Handles subscription and unsubscription to a topic
fn handle_subscription(req: &mut CoapRequest<SocketAddr>, topic_name: &str, subscriber_addr: SocketAddr, action: SubscriptionAction) {
    println!("Beginning subscription handling");

    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    let topic_collection_ref = Arc::get_mut(&mut locked_topic_collection).unwrap(); // Lock the topic map for safe access

    // Check if the topic exists
    if let Some(topic) = topic_collection_ref.find_topic_by_name_mut(topic_name) {
        if topic.half_created {
            // Topic does not exist, prepare an error response and respond that the subscibe action failed
            if let Some(ref mut message) = req.response {
            println!("{} tried to interact with {} but it failed because that topic is in half-created state", subscriber_addr.clone(), topic_name);
            message.message.payload = b"Topic not found".to_vec();
            message.set_status(coap_lite::ResponseType::NotFound);
            message.message.set_observe_value(1);
            return;
        }
        }
        let data = topic.get_data_resource();

        match action {
            SubscriptionAction::Subscribe => {
                // Topic exists, add subscriber
                data.add_subscriber(subscriber_addr.clone());
                println!("Current subscribers for {}: {:?}",topic_name.to_string(), data.get_subscribers());
                println!("{} subscribed to {}", subscriber_addr.clone().to_string(), topic_name);

                // Prepare a success response
                if let Some(ref mut message) = req.response {
                    // payload message just for testing purposes
                    //message.message.payload = b"Subscribed successfully".to_vec();
                    message.message.payload = data.get_data().clone().into_bytes().to_vec();
                    message.message.set_content_format(coap_lite::ContentFormat::try_from(110).unwrap());
                    message.message.set_observe_value(10001);
                }
            }
            SubscriptionAction::Unsubscribe => {
                // Topic exists, attempt to remove subscriber
                if data.get_subscribers().contains(&subscriber_addr) {
                    // Subscriber found, remove it
                    data.remove_subscriber(subscriber_addr.clone());
                    println!("{} unsubscribed from {}", subscriber_addr.clone(), topic_name);

                    // Prepare a success response
                    if let Some(ref mut message) = req.response {
                        message.message.payload = b"Unsubscribed successfully".to_vec();
                        message.message.set_observe_value(1);
                    }
                } 
                else {
                    // Subscriber not found, prepare an error response
                    if let Some(ref mut message) = req.response {
                        println!("{} tried to unsubsrcibe to {} but it failed because client isn't subscribed to that topic", subscriber_addr.clone(), topic_name);
                        message.message.payload = b"Subscriber not found".to_vec();
                        message.message.set_observe_value(1);
                    }
                }
            }
        }
    } 
    else {
        // Topic does not exist, prepare an error response and respond that the subscibe action failed
        if let Some(ref mut message) = req.response {
            println!("{} tried to unsubsrcibe to {} but it failed because no topic with that name exists", subscriber_addr.clone(), topic_name);
            message.message.payload = b"Topic not found".to_vec();
            message.set_status(coap_lite::ResponseType::NotFound);
            message.message.set_observe_value(1);
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
            handle_broker_discovery(req);
        },
        [topic, "subscribe"] => {
            handle_subscription(req, topic, req.source.unwrap(),SubscriptionAction::Subscribe);
        },
        [topic, "unsubscribe"] => {
            handle_subscription(req, topic, req.source.unwrap(),SubscriptionAction::Unsubscribe);
        },
        ["ps", "data", topic_data_uri] => {
            handle_get_latest_data(req, topic_data_uri);
        },
        [".well-known", "core?rt=core.ps.data"] => {
            handle_topic_data_discovery(req);
        }
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}

fn handle_topic_data_discovery(req: &mut CoapRequest<SocketAddr>) {
    println!("Handling topic data discovery");

    let locked_topic_collection = match TOPIC_COLLECTION_MUTEX.lock() {
        Ok(lock) => lock,
        Err(e) => {
            println!("Failed to lock TOPIC_COLLECTION_MUTEX: {}", e);
            return;
        }
    };

    let topic_collection = &*locked_topic_collection;

    let mut buffer = String::new();
    let mut write = LinkFormatWrite::new(&mut buffer);
    write.set_add_newlines(true);

    for topic in topic_collection.get_topics().values() {
        let data_resource = topic.get_dr();
        if data_resource.get_resource_type() == "core.ps.data" {
            write.link(&format!("/ps/{}", topic.get_topic_data()))
                 .attr(coap_lite::link_format::LINK_ATTR_RESOURCE_TYPE, "core.ps.data");
        }
    }

    if let Some(ref mut response) = req.response {
        response.message.payload = buffer.as_bytes().to_vec();
        response.set_status(coap_lite::ResponseType::Content);
        response.message.set_content_format(coap_lite::ContentFormat::ApplicationLinkFormat);
    } else {
        println!("Failed to set response payload");
    }
}






/// Handling put requests done to the broker
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
}

/// Updates data resource associated with a topic
async fn update_topic_data(req: &mut CoapRequest<SocketAddr>, topic_data: &str) {
    println!("{}",topic_data);
    let payload = match String::from_utf8(req.message.payload.clone()) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to decode payload as UTF-8");
            return;
        }
    };

    // Lock the mutex
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    let mut created = false;
    let mut updated = false;
    let topic: &mut Topic;
    // Obtain a mutable reference to the TopicCollection inside the Arc
    if let Some(topic_collection_ref) = Arc::get_mut(&mut locked_topic_collection) {
        // Attempt to find the topic by its topic_data
        if let Some(ttopic) = topic_collection_ref.find_topic_by_uri_mut(topic_data) {
            topic = ttopic;
            // Action is "data", update the topic's resource
            //If the topic has default data resource, make a new one and set it to the topic, and return 2.01 Created
            if topic.half_created == true {
                topic.get_data_resource().set_data(payload.to_string());
                topic.half_created = false;
                created = true;
                
            }
            // Otherwise, update the existing data resource and return 2.04 Updated
            else {
                topic.get_data_resource().set_data(payload.to_string());
                updated = true;
            }
        }
        else{
            println!("SETTING TOPIC DATA FAILED");
            if let Some(ref mut message)=req.response{
                notify_client(coap_lite::ResponseType::NotFound,message,"");
            }
            return;
        }
    }
    else{
        println!("Couldnt open topic collection");
        return;
    }

    // Notify all subscribers of the update
    for subscriber in topic.get_dr().get_subscribers() {
        // Clone the necessary data and move it into the async block
        let subscriber_clone = subscriber.clone();
        let resource = topic.get_dr().get_data().to_owned();

        println!("{}",subscriber_clone);
        tokio::spawn(async move {
            if let Err(e) = inform_subscriber(subscriber_clone, coap_lite::ResponseType::Changed, &resource).await {
                eprintln!("Failed to notify subscriber {}: {}", subscriber_clone, e);
            }
        });
    }

    if let Some(ref mut message) = req.response {
        if created {
            notify_client(coap_lite::ResponseType::Created, message, "Created");
        } else if updated {
            notify_client(coap_lite::ResponseType::Changed, message, "Updated");
        }
    } else {
        // Topic not found
        if let Some(ref mut message) = req.response {
            message.message.payload = b"Topic not found".to_vec();
        }
    }
}

async fn inform_subscriber(addr: SocketAddr, response_type: ResponseType, resource: &str) -> Result<(), Box<dyn std::error::Error>> {
    let packet = coap_lite::Packet::new();

    let mut message = CoapResponse::new(&packet).unwrap();
    message.set_status(response_type);
    message.message.payload = resource.as_bytes().to_vec();
    message.message.set_content_format(coap_lite::ContentFormat::try_from(110).unwrap());
    message.message.set_observe_value(10002);

    let payload = message.message.to_bytes().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.send_to(&payload, &addr)?;

    Ok(())
}

/// Creates a new topic
fn create_topic(topic_name: &String, resource_type: &String, req: &mut coap_lite::CoapRequest<SocketAddr>) {
    let topic = Topic::new(topic_name.clone(), resource_type.clone());
    let topic_uri = topic.get_topic_uri();
    let mut locked_topic_collection: std::sync::MutexGuard<'_, Arc<TopicCollection>> = TOPIC_COLLECTION_MUTEX.lock().unwrap();
    let mut topic_collection_ref = Arc::get_mut(&mut locked_topic_collection);
    topic_collection_ref.as_mut().unwrap().add_topic(topic);
    println!("Topic '{}' with uri: {}, and of type '{}' added to the topic map.", topic_name, topic_uri, resource_type);

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

/// Handles requests with method DELETE. 
async fn handle_delete(req: &mut CoapRequest<SocketAddr>) {
    let path = req.get_path(); // Extract the URI path from the request
    let components: Vec<&str> = path.split('/').filter(|c| !c.is_empty()).collect();

    match components.as_slice() {
        [topic_name] => {
            delete_topic(req, topic_name, req.source.unwrap());
        },
        _ => {
            // Handle invalid or unrecognized paths
            handle_invalid_path(req);
        },
    }
}
/// This function deletes a topic configuration as specified here: https://datatracker.ietf.org/doc/html/draft-ietf-core-coap-pubsub-13#name-deleting-a-topic-configurat
// TO DO: all subscribers MUST be unsubscribed after this
fn delete_topic(req: &mut CoapRequest<SocketAddr>, topic_uri: &str, local_addr: SocketAddr) {
    println!("Deleting topic: {}", topic_uri);
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap(); // Lock the topic map for safe access
    let mut topic_collection_ref = Arc::get_mut(&mut locked_topic_collection);
    
    topic_collection_ref.as_mut().unwrap().remove_topic(topic_uri);
        // Topic found and removed
        if let Some(ref mut message) = req.response {
            notify_client(coap_lite::ResponseType::Deleted, message, "Topic deleted succesfully");
            println!("{} deleted {}", local_addr.to_string(), topic_uri);
        }
}



fn handle_get_latest_data(req: &mut CoapRequest<SocketAddr>, topic_data_uri: &str) {
    // Lock the mutex to access the topic collection
    let mut locked_topic_collection = TOPIC_COLLECTION_MUTEX.lock().unwrap();

    let topic_collection_ref = Arc::get_mut(&mut locked_topic_collection).unwrap(); // Lock the topic map for safe access

    // Find the topic by its data URI
    if let Some(topic) = topic_collection_ref.find_topic_by_data_uri_mut(topic_data_uri) {
        // Check if the topic is fully created
        if topic.half_created {
            // Topic is not in fully created state, return 4.04 (Not Found)
            if let Some(ref mut message) = req.response {
                message.set_status(coap_lite::ResponseType::NotFound);
                message.message.payload = b"Topic data not found".to_vec();
            }
        } else {
            // Topic is fully created, return the latest data
            let data = topic.get_data_resource().get_data().clone(); // Assuming get_data() returns the latest data
            if let Some(ref mut message) = req.response {
                message.set_status(coap_lite::ResponseType::Content);
                message.message.payload = data.into_bytes().to_vec();
                message.message.set_content_format(coap_lite::ContentFormat::ApplicationJSON);
            }
        }
    } else {
        // Topic not found, return 4.04 (Not Found)
        if let Some(ref mut message) = req.response {
            message.set_status(coap_lite::ResponseType::NotFound);
            message.message.payload = b"Topic not found".to_vec();
        }
    }
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
    topic1.get_data_resource().set_data("123".to_string());
    topic1.set_topic_uri("123".to_string());
    topic1.set_topic_data(data_path1.clone());
    topic1.half_created = false;
    let mut topic2 = Topic::new("topic2".to_string(), "core.ps.conf".to_string());
    topic2.set_topic_uri("456".to_string());
    topic2.set_topic_data(data_path2.clone());
    topic2.half_created = false;
    let mut topic3 = Topic::new("topic3".to_string(), "core.ps.conf".to_string());
    topic3.set_topic_uri("789".to_string());
    topic3.set_topic_data(data_path3.clone());
    topic3.half_created = false;

    let mut data1 = DataResource::new();
    data1.set_data(example_data.to_string());
    //topic_collection.set_data(data_path1, data1);

    let mut data2 = DataResource::new();
    data2.set_data(example_data.to_string());
    //topic_collection.set_data(data_path2.clone(), data2);

    let mut data3 = DataResource::new();
    data3.set_data(example_data.to_string());
    //topic_collection.set_data(data_path3.clone(), data3);

    topic_collection.add_topic(topic1);
    topic_collection.add_topic(topic2);
    topic_collection.add_topic(topic3);
}

/// server startup and handling requests is implemented in main 
fn main() {
    initialize_topics();
    let addr = "127.0.0.1:5683";
    Runtime::new().unwrap().block_on(async move {
        // create socket2 socket and assign a random address to it, then join multicast group with it
        // and attempt to make these nonblocking and reusable
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(socket2::Protocol::UDP)).unwrap();
        socket.set_nonblocking(true).unwrap();
        socket.set_reuse_address(true).unwrap();
        let addr2 = "0.0.0.0:5683".parse::<std::net::SocketAddr>().unwrap();
        socket.bind(&addr2.into()).unwrap();
        // multicast address for ipv4 coap is 224.0.1.187:5683
        let multiaddr = Ipv4Addr::new(224, 0, 1, 187);
        socket.join_multicast_v4(&multiaddr, &Ipv4Addr::UNSPECIFIED).unwrap();

        // create std socket from socket2 socket and then tokio socket from std socket
        let sock = UdpSocket::from(socket);
        let socket_multi = tokio::net::UdpSocket::from_std(sock).unwrap();

        // and socket from 127.0.0.1:5683
        let socket_local = tokio::net::UdpSocket::bind(addr).await.unwrap();

        // create server from listeners
        let mut listeners: Vec<Box<dyn Listener>> = Vec::new();
        let listener1 = Box::new(UdpCoapListener::from_socket(socket_local));
        let listener2 =  Box::new(UdpCoapListener::from_socket(socket_multi));
        listeners.push(listener1);
        listeners.push(listener2);
        let mut server = Server::from_listeners(listeners);

        // remove basic functionality of handling get requests with observe setting
        server.disable_observe_handling(true).await;
        
        println!("Broker up on {}, listening for requests.", addr);

        // run the server and process requests
        server.run(|mut request: Box<CoapRequest<SocketAddr>>| async {
            match request.get_method() {
                &Method::Get => handle_get(&mut *request),
                &Method::Post => handle_post(&mut request),
                &Method::Put => handle_put(&mut *request).await,
                &Method::Delete => handle_delete(&mut *request).await,
                _ => println!("Error, request by method that is not supported."),
            };
            // respond to request
            return request;
        }).await.unwrap();
    });
}