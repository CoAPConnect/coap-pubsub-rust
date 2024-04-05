use std::net::SocketAddr;
use std::collections::HashMap;
use rand::Rng;


///Generate random len 6 String consisting of numbers and/or letters as the uri. 2,2 billion possibilities
///Currently doesn't check for possible same uris!! TODO
fn generate_uri() -> String {
    let mut rng = rand::thread_rng();

    let random_string: String = (0..6)
        .map(|_| {
            let choice = rng.gen_range(0..36);
            if choice < 10 {
                // Generate a digit
                (choice + b'0') as char
            } else {
                // Generate a letter
                (choice - 10 + b'a') as char
            }
        })
        .collect();
    random_string
}


///Topic resource as struct and its implemented methods.
///
///Fields are based on IETF draft <https://www.ietf.org/archive/id/draft-ietf-core-coap-pubsub-13.html>
///Referenced 25.3.2024
///
///Mandatory fields for topic creation: topic_name, resource_type (only "core.ps.conf" accepted).
///Optional fields for topic creation: topic_uri, topic_data, media_type, topic_type, expiration_date, max_subscribers.
/// Represents a topic in the broker.
pub struct Topic {
    /// The name of the topic.
    pub topic_name: String,
    /// The type of the resource associated with the topic.
    pub resource_type: String,
    /// The URI of the topic.
    pub topic_uri: String,
    /// The URI associated with the topic data
    pub topic_data: String,
    /// The media type of the topic data.
    pub media_type: String,
    /// The type of the topic.
    pub topic_type: String,
    /// The expiration date of the topic.
    pub expiration_date: String,
    /// The maximum number of subscribers allowed for the topic.
    pub max_subscribers: u32,
    /// The amount of time in seconds between each observer check, removing uninterested observers
    pub observe_check: u32,
    /// Data resource as part of topic
    pub data_resource: DataResource,
    /// State of the topic: half-created of fully created bool
    pub half_created: bool,
}

///Topic implementation.
///Create a new mutable struct for an example:
/// ```rust
/// let topic = Topic::new("topic_name".to_string(), "core.ps.conf".to_string());
/// ```
impl Topic {
    pub fn new(topic_name: String, resource_type: String) -> Self {
        Topic {
            topic_name,
            resource_type,
            topic_uri: generate_uri(),
            topic_data: generate_uri(),
            media_type: String::new(),
            topic_type: String::new(),
            expiration_date: String::new(),
            max_subscribers: u32::MAX,
            observe_check: 86400,
            data_resource: DataResource::new(),
            half_created: true,
        }
    }


    pub fn set_data_resource(&mut self, dr: DataResource){
        self.data_resource=dr;
    }
    pub fn get_data_resource(&mut self) -> &mut DataResource{
        &mut self.data_resource
    }
    pub fn get_dr (&self) -> &DataResource{
        &self.data_resource
    }


    ///Set the URI of the topic.
    pub fn set_topic_uri(&mut self, topic_uri: String) {
        self.topic_uri = topic_uri;
    }
    ///Set the data of the topic.
    pub fn set_topic_data(&mut self, topic_data: String) {
        self.topic_data = topic_data;
    }
    ///Set the media type of the topic data.
    pub fn set_media_type(&mut self, media_type: String) {
        self.media_type = media_type;
    }
    ///Set the type of the topic.
    pub fn set_topic_type(&mut self, topic_type: String) {
        self.topic_type = topic_type;
    }
    ///Set the expiration date of the topic.
    pub fn set_expiration_date(&mut self, expiration_date: String) {
        self.expiration_date = expiration_date;
    }
    ///Set the maximum number of subscribers allowed for the topic. Max value is determined by u32
    pub fn set_max_subscribers(&mut self, max_subscribers: u32) {
        if max_subscribers > u32::MAX{
            self.max_subscribers = u32::MAX;
        }else{
            self.max_subscribers = max_subscribers;
        }
    }
    ///Set the observe check time in seconds.
    pub fn set_observe_check(&mut self, observe_check: u32) {
        if observe_check > u32::MAX{
            self.observe_check = u32::MAX;
        }else{
            self.observe_check = observe_check;
        }
    }
    ///Get the name of the topic.
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }
    ///Get the type of the resource associated with the topic.
    pub fn get_resource_type(&self) -> &str {
        &self.resource_type
    }
    /// Get the URI of the topic.
    pub fn get_topic_uri(&self) -> String {
        self.topic_uri.clone()
    }
    ///Get the data of the topic.
    pub fn get_topic_data(&self) -> &str {
        &self.topic_data
    }
    ///Get the media type of the topic data.
    pub fn get_media_type(&self) -> &str {
        &self.media_type
    }
    ///Get the type of the topic.
    pub fn get_topic_type(&self) -> &str {
        &self.topic_type
    }
    ///Get the expiration date of the topic.
    pub fn get_expiration_date(&self) -> &str {
        &self.expiration_date
    }
    ///Get the maximum number of subscribers allowed for the topic.
    pub fn get_max_subscribers(&self) -> u32 {
        self.max_subscribers
    }
    ///Get the current observe check value
    pub fn get_observe_check(&self) -> u32 {
        self.observe_check
    }

}
///Topic collection as struct
pub struct TopicCollection {
    /// The name of the topic collection aka path for the collection "/name".
    name: String,
    /// The type of the resource associated with the topic collection "core.ps.coll".
    resource_type: String,
    /// The topics in the topic collection. Key: uri of the topic, Value: the topic assosiated with uri
    topics: HashMap<String, Topic>,
    /// Data for the topics with path/name String as key and value as String(json format)
    data: HashMap<String, DataResource>,
}
///Topic collection implementation.
impl TopicCollection {
    /// Creates a new topic collection with the specified name.
    pub fn new(name: String) -> Self {
        TopicCollection {
            name,
            resource_type: String::from("core.ps.coll"),
            topics: HashMap::new(),
            data: HashMap::new(),
        }
    }

    //Getters and setters

    /// Returns the name of the topic collection.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the resource type of the topic collection.
    pub fn get_resource_type(&self) -> &str {
        &self.resource_type
    }

    /// Returns the topics in the topic collection.
    pub fn get_topics(&self) -> &HashMap<String, Topic> {
        &self.topics
    }

    /// Returns reference to dataresource from path
    pub fn get_data_from_path(&self, path: String) -> &DataResource {
        println!("{}",path);
        for (key, value) in &self.data{
            println!("{}", key);
        }
        self.data.get(&path).clone().unwrap()
    }

    /// Returns a mutable reference to dataresource from path
    pub fn get_data_from_path_mut(&mut self, path: String) -> &mut DataResource {
        self.data.get_mut(&path).unwrap()
    }

    /// Returns data value from path, if path doesn't exist or no value, return empty String
    pub fn get_data_value_from_path(&self, path: String) -> String {
        self.data.get(&path).clone().unwrap().get_data().to_string()
    }

    /// Sets the name of the topic collection.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Sets the resource type of the topic collection.
    pub fn set_resource_type(&mut self, resource_type: String) {
        self.resource_type = resource_type;
    }

    /// Sets the topics in the topic collection.
    pub fn set_topics(&mut self, topics: HashMap<String, Topic>) {
        self.topics = topics;
    }

    /// Set new dataresource for a topic, inserts into hashmap
    pub fn set_data(&mut self, path: String, data: DataResource) {
        self.data.insert(path, data);
    }


    //Additional functionality

    /// Adds a topic to the topic collection.
    pub fn add_topic(&mut self, topic: Topic) {
        self.topics.insert(topic.get_topic_uri().to_string(), topic);
    }

    /// Removes a topic from the topic collection by its name.
    pub fn remove_topic(&mut self, topic_uri: &str) {
        self.topics.remove(topic_uri);
    }

    /// Finds a topic in the topic collection by its URI.
    pub fn find_topic_by_uri(&self, topic_uri: &str) -> Option<&Topic> {
        self.topics.get(topic_uri)
    }
  
    /// Find a topic in the collection by its topic_data URI
    pub fn find_topic_by_data_uri(&self, topic_data_uri: &str) -> Option<&Topic> {
        self.topics.values().find(|topic| topic.get_topic_data() == topic_data_uri)
    }
  
    //Find a topic in the collection by its data_uri and return mutable reference
    pub fn find_topic_by_data_uri_mut(&mut self, topic_data_uri: &str) -> Option<&mut Topic> {
        self.topics.values_mut().find(|topic| topic.get_topic_data() == topic_data_uri)
    }

    /// Finds a topic in the topic collection by its URI and returns it as mutable
    pub fn find_topic_by_uri_mut(&mut self, topic_uri: &str) -> Option<&mut Topic> {
        self.topics.get_mut(topic_uri)
    }

    /// Finds a topic in the topic collection by its name.
    pub fn find_topic_by_name(&self, topic_name: &str) -> Option<&Topic> {
        self.topics.values().find(|topic| topic.get_topic_name() == topic_name)
    }

    /// Finds a topic in the topic collection by its name and returns a mutable topic.
    pub fn find_topic_by_name_mut(&mut self, topic_name: &str) -> Option<&mut Topic> {
        self.topics.values_mut().find(|topic| topic.get_topic_name() == topic_name)
    }

    /// Changes current data in selected path, aka publishing new data if dataresource exists
    pub fn update_data_value(&mut self, path: String, value: String) {
        if let Some(data) = self.data.get_mut(&path) {
            data.data = value;
        }
    }
}
#[derive(Default)]
pub struct DataResource {
    data_uri: String,
    parent_topic_uri: String,
    resource_type: String,
    subscribers: Vec<SocketAddr>,
    data: String,
}

impl DataResource {
    pub fn new() -> Self {
        DataResource {
            data_uri: generate_uri(),
            parent_topic_uri: String::from("yolo"),
            resource_type: String::from("core.ps.data"),
            subscribers: Vec::new(),
            data: String::new(),
        }
    //Getters and setters
    }
    pub fn get_data_uri(&self) -> &str {
        &self.data_uri
    }
    pub fn get_parent_topic_uri(&self) -> &str {
        &self.parent_topic_uri
    }
    pub fn get_resource_type(&self) -> &str {
        &self.resource_type
    }
    pub fn get_subscribers(&self) -> &Vec<SocketAddr> {
        &self.subscribers
    }
    pub fn get_data(&self) -> &String {
        &self.data
    }
    pub fn set_data_uri(&mut self, data_uri: String) {
        self.data_uri = data_uri;
    }
    pub fn set_parent_topic_uri(&mut self, parent_topic_uri: String) {
        self.parent_topic_uri = parent_topic_uri;
    }
    pub fn set_resource_type(&mut self, resource_type: String) {
        self.resource_type = resource_type;
    }
    pub fn set_subscribers(&mut self, subscribers: Vec<SocketAddr>) {
        self.subscribers = subscribers;
    }
    pub fn set_data(&mut self, data: String) {
        self.data = data;
    }
    //Adding and removing subscribers
    pub fn add_subscriber(&mut self, subscriber: SocketAddr) {
        self.subscribers.push(subscriber);
    }
    pub fn remove_subscriber(&mut self, subscriber: SocketAddr) {
        self.subscribers.retain(|s| s != &subscriber);
    }
}