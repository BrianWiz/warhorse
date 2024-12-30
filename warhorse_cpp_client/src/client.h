#pragma once

#include <string>
#include <vector>
#include <functional>
#include "include/bindings.h"

enum MessageType {
    HELLO,
    LOGGED_IN,
    ERROR,
    FRIEND_REQUESTS,
    FRIENDS_LIST,
    BLOCKED_LIST,
    FRIEND_REQUEST_ACCEPTED,
    CHAT_MESSAGE
};

struct Message {
    MessageType type;
    std::string message;
};

typedef std::function<void(const char*)> WarhorseCallback;

static void log(const std::string& message);

class WarhorseClient {
public:
    WarhorseClient(const std::string& connection_string);
    ~WarhorseClient();
    bool login(const std::string& username, const std::string& password);
    bool pump_messages(std::vector<Message>& messages);
    bool is_ready_for_login() const;

    // Callbacks
    WarhorseCallback on_hello;
    WarhorseCallback on_logged_in;
    WarhorseCallback on_error;
    WarhorseCallback on_friend_requests;
    WarhorseCallback on_friends_list;
    WarhorseCallback on_blocked_list;
    WarhorseCallback on_friend_request_accepted;
    WarhorseCallback on_chat_message;

    // Binds to callbacks
    void bind_on_hello(WarhorseCallback cb) { on_hello = cb; }
    void bind_on_logged_in(WarhorseCallback cb) { on_logged_in = cb; }
    void bind_on_error(WarhorseCallback cb) { on_error = cb; }
    void bind_on_friend_requests(WarhorseCallback cb) { on_friend_requests = cb; }
    void bind_on_friends_list(WarhorseCallback cb) { on_friends_list = cb; }
    void bind_on_blocked_list(WarhorseCallback cb) { on_blocked_list = cb; }
    void bind_on_friend_request_accepted(WarhorseCallback cb) { on_friend_request_accepted = cb; }
    void bind_on_chat_message(WarhorseCallback cb) { on_chat_message = cb; }
private:
    bool logged_in;
    bool received_hello;
    bool sent_login_request;
    warhorse::WarhorseClientHandle* handle;
};