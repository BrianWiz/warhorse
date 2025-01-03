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

# CRATES (all highly WIP)
## warhorse_client
The Rust client lib. This is a lib crate that contains the client logic.

## warhorse_server
The Rust server. This is a bin crate that starts a server that listens for incoming connections and messages.

## warhorse_protocol
The Rust library. It contains the protocol definitions and the message types that are used by the client and server.

## warhorse_overlay
Dioxus app that will be used to overlay the social GUI on top of a game. This will be used to display the friends list, chat, and other features.

## warhorse_experimentation
Dumping ground for experimentation. This is where I test out new ideas and concepts.

## warhorse_cpp_client
The C++ client. It contains an example of a simple client that connects to the server and sends a message.

## warhorse_cpp
The C++ bindings.
