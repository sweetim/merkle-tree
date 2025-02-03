## Merkle Tree App

This repository contains a Rust implementation of a Merkle Tree, a data structure commonly used for data integrity verification.  It also provides functionalities for searching the tree, displaying the tree in different formats, and generating a Merkle path for a specific data element.

### Merkle root lib

A rust library that provides the merkle tree algorithm

### Proof of reserve app

A REST API server that exposes 3 API

| endpoint         | description                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------- |
| /proof           | Displays the merkle root of the data                                                              |
| /proof/mermaid   | Displays the Merkle Tree as a Mermaid diagram and view it at (mermaid)[https://mermaid.live/edit] |
| /proof/<user-id> | Searches for a user with the given ID and display the proof                                       |



## Getting Started

### Prerequisites

*   Rust and Cargo (`cargo 1.84.0`) installed.

### Usage of proof of reserve app

To run the proof of reserve app, use the following command:

```
cargo run --release
```

