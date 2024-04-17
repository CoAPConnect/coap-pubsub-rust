# coap-pubsub-rust

This is an implementation of a simple CoAP pubsub broker and client for testing. Included also is an implementation of required resource types. The implementation is done according to the IETF draft [A publish-subscribe architecture for the Constrained Application Protocol (CoAP) ](https://datatracker.ietf.org/doc/draft-ietf-core-coap-pubsub/) version 13.

This project was a part of the Software Engineering project course at Tampere University.

# Installation

Make sure you have installed Rust and cargo. [Rust installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Clone the git repo locally

# Usage

Once you have cloned the repo, navigate to the src/broker file and run cargo run

```
cd path_to_src/broker && cargo run
```
Next, open a new terminal window and run the client
```
cd path_to_src/client && cargo run
```

The client terminal gives an interface to test the broker with. You can optionally test the broker with any other working CoAP client.
//TO DO WRITE MORE ON USAGE

# Resource

This implements the resource classes required for publish subscribe directly. These should be replaced in future work once the base Rust CoAP libraries get updated to have extensible resource implementations.

// # Contributors

our names, tuni

// # Supported operations
// What operations the broker supports with respect to the draft
// - [ ] top level
//  - [ ] mid level
//  - [x] midlevel
