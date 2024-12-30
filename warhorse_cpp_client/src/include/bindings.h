#ifndef WARHORSE_H
#define WARHORSE_H

#include <cstddef>
#include <cstdint>

namespace warhorse {

enum class WarhorseEventType {
  Hello,
  LoggedIn,
  Error,
  FriendRequests,
  FriendsList,
  BlockedList,
  FriendRequestAccepted,
  ChatMessage,
};

struct WarhorseClientHandle {
  uint8_t _private;
};

struct WarhorseEventData {
  WarhorseEventType event_type;
  char *message;
};

extern "C" {

void use_log();

WarhorseClientHandle *client_new(const char *connection_string);

bool client_login_with_username(WarhorseClientHandle *handle,
                                const char *username,
                                const char *password);

uintptr_t client_pump(WarhorseClientHandle *handle,
                      WarhorseEventData *events,
                      uintptr_t max_events);

void log_info(const char *message);

void log_error(const char *message);

void client_free(WarhorseClientHandle *handle);

void free_string(char *ptr);

}  // extern "C"

}  // namespace warhorse

#endif  // WARHORSE_H
