use std::net::SocketAddr;
pub struct DataResource {
    data_uri: String,
    parent_topic_uri: String,
    resource_type: String,
    subscibers: Vec<SocketAddr>,
}

impl DataResource {
    pub fn new(data_uri: String, parent_topic_uri: String) -> Self {
        DataResource {
            data_uri,
            parent_topic_uri,
            resource_type: String::from("core.ps.data"),
            subscibers: Vec::new(),
        }
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
        &self.subscibers
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
        self.subscibers = subscribers;
    }
    pub fn add_subscriber(&mut self, subscriber: SocketAddr) {
        self.subscibers.push(subscriber);
    }
    pub fn remove_subscriber(&mut self, subscriber: SocketAddr) {
        self.subscibers.retain(|s| s != &subscriber);
    }
    /*
    Additional functionality
    Check if data changes -> ping subscribers?
    Link parent URI to data URI?
     */
}