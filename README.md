# coap-pubsub-rust

This is an implementation of a simple CoAP pubsub broker and client for testing. Included also is an implementation of required resource types. The implementation is done according to the IETF draft [A publish-subscribe architecture for the Constrained Application Protocol (CoAP) ](https://datatracker.ietf.org/doc/draft-ietf-core-coap-pubsub/) version 13.

This project was a part of the Software Engineering project course at Tampere University. This implementation is heavily influenced by the Python aiocoap implementation: [https://github.com/jaimejim/aiocoap-pubsub-broker/](https://github.com/jaimejim/aiocoap-pubsub-broker/)

## Installation

Make sure you have installed Rust and cargo. [Rust installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Clone the git repo locally

## Usage

Once you have cloned the repo, navigate to the src/broker file and run cargo run

```
cd path_to_folder/broker && cargo run
```
Next, open a new terminal window and run the client
```
cd path_to_folder/client && cargo run
```

The client terminal gives an interface to test the broker with. You can optionally test the broker with any other working CoAP client.

Once you have both the broker and client running, you should have views somewhat like this:

Broker:

![image](https://github.com/CoAPConnect/coap-pubsub-rust/assets/80271133/b06a6565-7c00-41a5-83c3-b1fc7ab567a0)

Client:

![image](https://github.com/CoAPConnect/coap-pubsub-rust/assets/80271133/8a83c710-1453-4ac1-98a1-dad989c48576)

The client side accepts commands, write them in and press enter to execute.

### Topic-Configuration Discovery
Write
```
11
```
In the client side and hit enter, the response should be a collection of the topic-configurations

### Topic-Data Discovery
Write
```
12
```
In the client side and hit enter, the response should be a collection of the topic-data paths. 

### Topic-Collection Discovery
Write
```
13
```
In the client side and hit enter, the response is the ps collection. The current state only supports the ps collection.

### Broker Discovery
Write
```
8
```
In the client side and hit enter, the response is the response to doing a get to .well-known/core. You can repeat the same for multicast with command 7

### Topic creation

To create a topic, you need a topic name. The current version does not support adding in max-observers, observer-check, expiration or media-type. The resource-type of "core.ps.conf" is hardcoded in. To create a topic, use:

```
4 <Topic Name>
```

### Publish

Publication requires you to know the data-uri of the topic-data you want to publish to. Since only one topic-collection is possible, we have the base path of ps/data hardcoded in. To publish, use:

```
5 <DataUri> <Value>
```

### Subscribe

To subscribe, you need to know the data-uri too. Use:

```
2 <DataUri>
```

### Unsubscribe
Similar to subscription:

```
3 DataUri
```
### Topic Deletion

We need the topic-uri to delete a topic configuration. The topic deletion deletes the topic from the broker and informs subscribers with a final 4.04. Use:

```
6 <TopicUri>
```

#### Topic name/uri/datauri discovery

This is not a draft specified functionality, but is used for quickly checking the contents of the broker, just type 
```
1
```
and hit enter.

## Run integration tests

Turn the broker on first, and then go to test-client folder and run 

```
cargo test
```
This tests a very basic workflow. 

## Note on limits of the current state of the project

The topic configuration storage only supports the topic-uri, name and resource-type. Therefore observe-checks, checking for max-subscribers, expiration and media-type aren't implemented.

The current default content-type is application/json, did not get around to running CBOR.

Any functionalities requiring PATCH or FETCH do not work, since the underlying libraries do not have them implemented.





<!---
//TO DO WRITE MORE ON USAGE

# Resource

This implements the resource classes required for publish subscribe directly. These should be replaced in future work once the base Rust CoAP libraries get updated to have extensible resource implementations.



// # Supported operations
// What operations the broker supports with respect to the draft
// - [ ] top level
//  - [ ] mid level
//  - [x] midlevel
--->
