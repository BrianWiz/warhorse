# Warhorse

## Planned Features (for now)
- Rust client
  - Bevy example frontend
- C++ client
- separation of account name and display name
- live friends list update including user online status/presence
- real-time chat with private messages and rooms (channels)
- live game sessions
  - attach arbitrary data like player scores
  - game session search

## Compiling & Running
- in one tab: `cargo run horse_server`
- in another tab: `cargo run horse_client`

# CRATES (all highly WIP)
## warhorse_client
The Rust client. It contains an example of a simple client that connects to the server and sends a message.

## warhorse_server
The Rust server. This is a bin crate that starts a server that listens for incoming connections and messages.

## warhorse_protocol
The Rust library. It contains the protocol definitions and the message types that are used by the client and server.

## warhorse_ui and warhorse_ui_experiments
Experimental GUI for Bevy.

## warhorse_cpp_client
The C++ client. It contains an example of a simple client that connects to the server and sends a message.

## warhorse_cpp
The C++ bindings.
