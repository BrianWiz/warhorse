#include "client.h"

#include <iostream>
#include <thread>

WarhorseClient::WarhorseClient(const std::string& connection_string) {

    logged_in = false;
    received_hello = false;
    sent_login_request = false;

    on_hello = nullptr;
    on_logged_in = nullptr;
    on_error = nullptr;
    on_friend_requests = nullptr;
    on_friends_list = nullptr;
    on_blocked_list = nullptr;
    on_friend_request_accepted = nullptr;
    on_chat_message = nullptr;

    handle = warhorse::client_new(connection_string.c_str());
}

WarhorseClient::~WarhorseClient() {
    warhorse::client_free(handle);
}

bool WarhorseClient::login(const std::string& username, const std::string& password) {

    if (warhorse::client_login_with_username(handle, username.c_str(), password.c_str()))
    {
        sent_login_request = true;
        return true;
    }

    return false;
}

bool WarhorseClient::pump_messages(std::vector<Message>& messages) {

    constexpr size_t MAX_EVENTS = 32;
    warhorse::WarhorseEventData events[MAX_EVENTS];

    size_t event_count = warhorse::client_pump(handle, events, MAX_EVENTS);
    for (size_t i = 0; i < event_count; i++) {
        Message message;
        switch (events[i].event_type) {
            case warhorse::WarhorseEventType::Hello:
                message.type = HELLO;
                break;
            case warhorse::WarhorseEventType::LoggedIn:
                message.type = LOGGED_IN;
                logged_in = true;
                break;
            case warhorse::WarhorseEventType::Error:
                message.type = ERROR;
                break;
            case warhorse::WarhorseEventType::FriendRequests:
                message.type = FRIEND_REQUESTS;
                break;
            case warhorse::WarhorseEventType::FriendsList:
                message.type = FRIENDS_LIST;
                break;
            case warhorse::WarhorseEventType::BlockedList:
                message.type = BLOCKED_LIST;
                break;
            case warhorse::WarhorseEventType::FriendRequestAccepted:
                message.type = FRIEND_REQUEST_ACCEPTED;
                break;
            case warhorse::WarhorseEventType::ChatMessage:
                message.type = CHAT_MESSAGE;
                break;
        }

        message.message = events[i].message;
        messages.push_back(message);
    }

    return event_count > 0;
}

bool WarhorseClient::is_ready_for_login() const {
    return received_hello && !sent_login_request;
}

int test_warhorse_client() {
    warhorse::use_log();

    auto client = std::make_shared<WarhorseClient>("http://localhost:3000");

    std::weak_ptr<WarhorseClient> weak_client = client;

    client->bind_on_hello([weak_client](const char* message) {
        if (auto c = weak_client.lock()) {
            c->login("test", "password");
        }
    });

    client->bind_on_logged_in([](const char* message) {

    });

    client->bind_on_error([](const char* message) {

    });

    client->bind_on_friend_requests([](const char* message) {

    });

    client->bind_on_friends_list([](const char* message) {

    });

    client->bind_on_blocked_list([](const char* message) {

    });

    client->bind_on_friend_request_accepted([](const char* message) {

    });

    client->bind_on_chat_message([](const char* message) {

    });

    bool exit = false;

    while (exit == false) {
        if (client->is_ready_for_login()) {
        }

        std::vector<Message> messages;
        if (client->pump_messages(messages)) {
            for (const auto& message : messages) {
                switch (message.type) {
                    case HELLO:
                        if (client->on_hello) client->on_hello(message.message.c_str());
                        break;
                    case LOGGED_IN:
                        if (client->on_logged_in) client->on_logged_in(message.message.c_str());
                        break;
                    case ERROR:
                        if (client->on_error) client->on_error(message.message.c_str());
                        break;
                    case FRIEND_REQUESTS:
                        if (client->on_friend_requests) client->on_friend_requests(message.message.c_str());
                        break;
                    case FRIENDS_LIST:
                        if (client->on_friends_list) client->on_friends_list(message.message.c_str());
                        break;
                    case BLOCKED_LIST:
                        if (client->on_blocked_list) client->on_blocked_list(message.message.c_str());
                        break;
                    case FRIEND_REQUEST_ACCEPTED:
                        if (client->on_friend_request_accepted) client->on_friend_request_accepted(message.message.c_str());
                        break;
                    case CHAT_MESSAGE:
                        if (client->on_chat_message) client->on_chat_message(message.message.c_str());
                        break;
                }
            }
        }

        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }

    return 0;
}

int main() {
    return test_warhorse_client();
}
