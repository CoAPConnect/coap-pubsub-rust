use std::collections::HashMap;

// implements coap resources, link format etc

pub struct CoapResource {
    uri: String,
    resource_type: String,
    // maybe expand into attributes list?
    attributes: HashMap<String, String>,
}