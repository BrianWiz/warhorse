use std::ffi::{c_char, CStr, CString};
use serde_json::Value;
use tracing::{error, info};
use warhorse_client::{WarhorseClient, WarhorseEvent};
use warhorse_client::warhorse_protocol::*;

struct WarhorseClientImpl(Box<WarhorseClient>);

#[repr(C)]
pub struct WarhorseClientHandle {
    _private: u8
}

#[repr(C)]
pub enum WarhorseEventType {
    Hello,
    LoggedIn,
    Error,
    FriendRequests,
    FriendsList,
    BlockedList,
    FriendRequestAccepted,
    ChatMessage,
}

#[repr(C)]
pub struct WarhorseEventData {
    pub event_type: WarhorseEventType,
    pub message: *mut c_char,  // Will contain JSON string for complex data
}

#[no_mangle]
pub extern "C" fn use_log() {
    tracing_subscriber::fmt::init();
}

#[no_mangle]
pub extern "C" fn client_new(connection_string: *const c_char) -> *mut WarhorseClientHandle {
    let connection_str = unsafe {
        match CStr::from_ptr(connection_string).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    match WarhorseClient::new(connection_str) {
        Ok(client) => {
            let impl_handle = Box::new(WarhorseClientImpl(Box::new(client)));
            Box::into_raw(impl_handle) as *mut WarhorseClientHandle
        }
        Err(_) => std::ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn client_login_with_username(
    handle: *mut WarhorseClientHandle,
    username: *const c_char,
    password: *const c_char
) -> bool {
    let handle = unsafe {
        if handle.is_null() {
            lerror("Null handle passed to login");
            return false;
        }
        &*(handle as *mut WarhorseClientImpl)
    };

    let username_str = unsafe {
        match CStr::from_ptr(username).to_str() {
            Ok(s) => s,
            Err(e) => {
                lerror(&format!("Error converting username to string {}", e));
                return false;
            },
        }
    };

    let password_str = unsafe {
        match CStr::from_ptr(password).to_str() {
            Ok(s) => s,
            Err(e) => {
                lerror(&format!("Error converting password to string {}", e));
                return false;
            },
        }
    };

    let login = UserLogin {
        language: Language::English,
        identity: LoginUserIdentity::AccountName(username_str.to_string()),
        password: password_str.to_string(),
    };

    match handle.0.send_user_login_request(login) {
        Ok(_) => {
            linfo("Attempting to login to Warhorse");
            true
        },
        Err(e) => {
            lerror(&format!("Error logging in: {}", e));
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn client_pump(
    handle: *mut WarhorseClientHandle,
    events: *mut WarhorseEventData,
    max_events: usize,
) -> usize {
    let handle = unsafe {
        if handle.is_null() { return 0; }
        &*(handle as *mut WarhorseClientImpl)
    };

    let rust_events = handle.0.pump();
    let mut count = 0;

    for (i, event) in rust_events.into_iter().take(max_events).enumerate() {
        let event_data = unsafe {
            &mut *events.add(i)
        };

        match event {
            WarhorseEvent::Hello => {
                linfo("Received hello event");
                event_data.event_type = WarhorseEventType::Hello;
                match to_json_as_cstring(&Value::Null) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing hello message: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::LoggedIn => {
                linfo("Received logged in event");
                event_data.event_type = WarhorseEventType::LoggedIn;
                match to_json_as_cstring(&Value::Null) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing logged in message: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::Error(msg) => {
                linfo(&format!("Received error event: {:?}", msg).as_str());
                event_data.event_type = WarhorseEventType::Error;
                match to_json_as_cstring(&msg) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing error message: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::FriendRequests(friends) => {
                linfo(&format!("Received friend requests event: {:?}", friends).as_str());
                event_data.event_type = WarhorseEventType::FriendRequests;
                match to_json_as_cstring(&friends) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing friend requests: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::FriendsList(friends) => {
                linfo(&format!("Received friends list event: {:?}", friends).as_str());
                event_data.event_type = WarhorseEventType::FriendsList;
                match to_json_as_cstring(&friends) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing friends list: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::BlockedList(friends) => {
                linfo(&format!("Received blocked list event: {:?}", friends).as_str());
                event_data.event_type = WarhorseEventType::BlockedList;
                match to_json_as_cstring(&friends) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing blocked list: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::FriendRequestAccepted(friend) => {
                linfo(&format!("Received friend request accepted event: {:?}", friend).as_str());
                event_data.event_type = WarhorseEventType::FriendRequestAccepted;
                match to_json_as_cstring(&friend) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing friend request accepted: {}", e));
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
            WarhorseEvent::ChatMessage(msg) => {
                linfo(&format!("Received chat message event: {:?}", msg).as_str());
                event_data.event_type = WarhorseEventType::ChatMessage;
                match to_json_as_cstring(&msg) {
                    Ok(cstr) => event_data.message = cstr.into_raw(),
                    Err(e) => {
                        lerror(&format!("Error serializing chat message: {}", e).as_str());
                        event_data.message = std::ptr::null_mut()
                    },
                }
            }
        }
        count += 1;
    }
    count
}

fn to_json_as_cstring<T: serde::Serialize>(value: &T) -> Result<CString, String> {
    serde_json::to_string(value)
        .map_err(|e| e.to_string())
        .and_then(|json_str| {
            CString::new(json_str)
                .map_err(|e| e.to_string())
        })
}

fn linfo(message: &str) {
    let c_str = match CString::new(message) {
        Ok(s) => s,
        Err(_) => return,
    };
    log_info(c_str.as_ptr());
}

fn lerror(message: &str) {
    let c_str = match CString::new(message) {
        Ok(s) => s,
        Err(_) => return,
    };
    log_error(c_str.as_ptr());
}

#[no_mangle]
pub extern "C" fn log_info(message: *const c_char) {
    let message = unsafe {
        match CStr::from_ptr(message).to_str() {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    if tracing::level_enabled!(tracing::Level::INFO) {
        info!("{}", message);
    } else {
        println!("{}", message);
    }
}

#[no_mangle]
pub extern "C" fn log_error(message: *const c_char) {
    let message = unsafe {
        match CStr::from_ptr(message).to_str() {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    if tracing::level_enabled!(tracing::Level::ERROR) {
        error!("{}", message);
    } else {
        eprintln!("{}", message);
    }
}

#[no_mangle]
pub extern "C" fn client_free(handle: *mut WarhorseClientHandle) {
    unsafe {
        if !handle.is_null() {
            let _ = Box::from_raw(handle as *mut WarhorseClientImpl);
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}