/// #Topic resource as struct and its implemented methods.
///
/// Fields are based on IETF draft <https://www.ietf.org/archive/id/draft-ietf-core-coap-pubsub-13.html>
/// Referenced 25.3.2024
///
/// Mandatory fields for topic creation: topic_name, resource_type (only "core.ps.conf" accepted).
/// Optional fields for topic creation: topic_uri, topic_data, media_type, topic_type, expiration_date, max_subscribers.

pub struct Topic {
    /// The name of the topic.
    pub topic_name: String,
    /// The resource type of the topic. Only "core.ps.conf" is accepted.
    pub resource_type: String,
    /// The URI of the topic. Optional field.
    pub topic_uri: Option<String>,
    /// The data associated with the topic. Optional field.
    pub topic_data: Option<String>,
    /// The media type of the topic. Optional field.
    pub media_type: Option<String>,
    /// The type of the topic. Optional field.
    pub topic_type: Option<String>,
    /// The expiration date of the topic. Optional field.
    pub expiration_date: Option<String>,
    /// The maximum number of subscribers allowed for the topic.
    pub max_subscribers: i32,
}

/// #Topic implementation.
/// Create a new mutable struct for an example:
///```rust
///let mut topic = Topic::new(String::from("topic1"), String::from("core.ps.conf"));
///```
impl Topic {
    /// Creates a new topic with the specified name and resource type.
    pub fn new(topic_name: String, resource_type: String) -> Self {
        Topic {
            topic_name,
            resource_type,
            topic_uri: None,
            topic_data: None,
            media_type: None,
            topic_type: None,
            expiration_date: None,
            max_subscribers: 86400,
        }
    }

    /// Returns the name of the topic.
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Returns the resource type of the topic.
    pub fn get_resource_type(&self) -> &str {
        &self.resource_type
    }

    /// Returns the URI of the topic, if available.
    pub fn get_topic_uri(&self) -> Option<&str> {
        self.topic_uri.as_deref()
    }

    /// Returns the data associated with the topic, if available.
    pub fn get_topic_data(&self) -> Option<&str> {
        self.topic_data.as_deref()
    }

    /// Returns the media type of the topic, if available.
    pub fn get_media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }

    /// Returns the type of the topic, if available.
    pub fn get_topic_type(&self) -> Option<&str> {
        self.topic_type.as_deref()
    }

    /// Returns the expiration date of the topic, if available.
    pub fn get_expiration_date(&self) -> Option<&str> {
        self.expiration_date.as_deref()
    }

    /// Returns the maximum number of subscribers allowed for the topic.
    pub fn get_max_subscribers(&self) -> i32 {
        self.max_subscribers
    }

    /// Sets the name of the topic.
    pub fn set_topic_name(&mut self, topic_name: String) {
        self.topic_name = topic_name;
    }

    /// Sets the resource type of the topic.
    pub fn set_resource_type(&mut self, resource_type: String) {
        self.resource_type = resource_type;
    }

    /// Sets the URI of the topic.
    pub fn set_topic_uri(&mut self, topic_uri: String) {
        self.topic_uri = Some(topic_uri);
    }

    /// Sets the data associated with the topic.
    pub fn set_topic_data(&mut self, topic_data: String) {
        self.topic_data = Some(topic_data);
    }

    /// Sets the media type of the topic.
    pub fn set_media_type(&mut self, media_type: String) {
        self.media_type = Some(media_type);
    }

    /// Sets the type of the topic.
    pub fn set_topic_type(&mut self, topic_type: String) {
        self.topic_type = Some(topic_type);
    }

    /// Sets the expiration date of the topic.
    pub fn set_expiration_date(&mut self, expiration_date: String) {
        self.expiration_date = Some(expiration_date);
    }

    /// Sets the maximum number of subscribers allowed for the topic.
    pub fn set_max_subscribers(&mut self, max_subscribers: i32) {
        self.max_subscribers = max_subscribers;
    }
}

pub struct TopicCollection {
    name: String,
    resource_type: String,
    topics: Vec<Topic>,
}

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
        self.topics.iter().find(|topic| topic.get_topic_uri() == Some(topic_uri))
    }

    /// Finds a topic in the topic collection by its name.
    pub fn find_topic_by_name(&self, topic_name: &str) -> Option<&Topic> {
        self.topics.iter().find(|topic| topic.get_topic_name() == topic_name)
    }
}