///#Topic resource as struct and its implemented methods.
///
///Fields are based on IETF draft https://www.ietf.org/archive/id/draft-ietf-core-coap-pubsub-13.html
///Referenced 25.3.2024
///
///Mandatory fields for topic creation: topic_name, resource_type (only "core.ps.conf" accepted).
///Optional fields for topic creation: topic_uri, topic_data, media_type, topic_type, expiration_date, max_subscribers.
pub struct Topic {
    topic_name: String,
    resource_type: String,
    topic_uri: Option<String>,
    topic_data: Option<String>,
    media_type: Option<String>,
    topic_type: Option<String>,
    expiration_date: Option<String>,
    max_subscribers: i32,
}
///#Topic implementation.
///Create a new mutable struct for an example:
///'''let mut topic = Topic::new(String::from("topic1"), String::from("core.ps.conf"));'''
impl Topic {
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
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }
    pub fn get_resource_type(&self) -> &str {
        &self.resource_type
    }
    pub fn get_topic_uri(&self) -> Option<&str> {
        self.topic_uri.as_deref()
    }
    pub fn get_topic_data(&self) -> Option<&str> {
        self.topic_data.as_deref()
    }
    pub fn get_media_type(&self) -> Option<&str> {
        self.media_type.as_deref()
    }
    pub fn get_topic_type(&self) -> Option<&str> {
        self.topic_type.as_deref()
    }
    pub fn get_expiration_date(&self) -> Option<&str> {
        self.expiration_date.as_deref()
    }
    pub fn get_max_subscribers(&self) -> i32 {
        self.max_subscribers
    }
    pub fn set_topic_name(&mut self, topic_name: String) {
        self.topic_name = topic_name;
    }
    pub fn set_resource_type(&mut self, resource_type: String) {
        self.resource_type = resource_type;
    }
    pub fn set_topic_uri(&mut self, topic_uri: String) {
        self.topic_uri = Some(topic_uri);
    }
    pub fn set_topic_data(&mut self, topic_data: String) {
        self.topic_data = Some(topic_data);
    }
    pub fn set_media_type(&mut self, media_type: String) {
        self.media_type = Some(media_type);
    }
    pub fn set_topic_type(&mut self, topic_type: String) {
        self.topic_type = Some(topic_type);
    }
    pub fn set_expiration_date(&mut self, expiration_date: String) {
        self.expiration_date = Some(expiration_date);
    }
    pub fn set_max_subscribers(&mut self, max_subscribers: i32) {
        self.max_subscribers = max_subscribers;
    }
}