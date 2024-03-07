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
    /// The data associated with the topic.
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
            topic_uri: String::new(),
            topic_data: String::new(),
            media_type: String::new(),
            topic_type: String::new(),
            expiration_date: String::new(),
            max_subscribers: u32::MAX,
            observe_check: 86400,
        }
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
    ///Get the URI of the topic.
    pub fn get_topic_uri(&self) -> &str {
        &self.topic_uri
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
    /// The name of the topic collection.
    name: String,
    /// The type of the resource associated with the topic collection "core.ps.coll".
    resource_type: String,
    /// The topics in the topic collection.
    topics: Vec<Topic>,
}
///Topic collection implementation.
impl TopicCollection {
    /// Creates a new topic collection with the specified name.
    pub fn new(name: String) -> Self {
        TopicCollection {
            name,
            resource_type: String::from("core.ps.coll"),
            topics: Vec::new(),
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
    pub fn get_topics(&self) -> &Vec<Topic> {
        &self.topics
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
    pub fn set_topics(&mut self, topics: Vec<Topic>) {
        self.topics = topics;
    }

    //Additional functionality

    /// Adds a topic to the topic collection.
    pub fn add_topic(&mut self, topic: Topic) {
        self.topics.push(topic);
    }

    /// Removes a topic from the topic collection by its name.
    pub fn remove_topic(&mut self, topic_name: &str) {
        self.topics.retain(|topic| topic.topic_name != topic_name);
    }

    /// Finds a topic in the topic collection by its URI.
    pub fn find_topic_by_uri(&self, topic_uri: &str) -> Option<&Topic> {
        self.topics.iter().find(|topic| topic.get_topic_uri() == topic_uri)
    }

    /// Finds a topic in the topic collection by its name.
    pub fn find_topic_by_name(&self, topic_name: &str) -> Option<&Topic> {
        self.topics.iter().find(|topic| topic.get_topic_name() == topic_name)
    }

    /// Finds a topic in the topic collection by its name and returns a mutable topic.
    pub fn find_topic_by_name_mut(&mut self, topic_name: &str) -> Option<&mut Topic> {
        self.topics.iter_mut().find(|topic| topic.get_topic_name() == topic_name)
    }
}