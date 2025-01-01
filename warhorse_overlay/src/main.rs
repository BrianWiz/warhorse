mod ui;

use std::sync::{Arc, Mutex};

use dioxus::desktop::tao::event::Event;
use dioxus::desktop::tao::keyboard::KeyCode;
use dioxus::desktop::WindowEvent;
use dioxus::desktop::{tao::platform::windows::WindowBuilderExtWindows, Config, WindowBuilder};
use tracing::{error, info};
use warhorse_client::WarhorseClient;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::{
        Input::KeyboardAndMouse::{EnableWindow, SetFocus},
        WindowsAndMessaging::{
            EnumWindows, FindWindowExW, GetWindowLongPtrW, GetWindowTextW, SetWindowLongPtrW,
            GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP,
            WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
        },
    },
};

const MAIN_WINDOW_TITLE: &str = "Visual Studio Code";
const OVERLAY_WINDOW_TITLE: &str = "Warhorse Game Overlay";
static mut OVERLAY_VISIBLE: bool = true;
static mut HOTKEY_WAS_PRESSED: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let overlay_mode = false; // Toggle this for overlay vs normal window

    match WarhorseClient::new("http://localhost:3000") {
        Ok(client) => {
            if overlay_mode {
                start_overlay(client);
            } else {
                start_normal(client);
            }
        }
        Err(e) => error!("Failed to initialize Warhorse client: {:?}", e),
    }
    Ok(())
}

fn start_normal(client: WarhorseClient) {
    dioxus::LaunchBuilder::desktop()
        .with_context(Arc::new(Mutex::new(client)))
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("Warhorse")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 720.0)),
            ),
        )
        .launch(ui::components::app);
}

fn start_overlay(client: WarhorseClient) {
    if let Ok(hwnd) = find_game_window() {
        info!("Game window found: {:?}", hwnd);
        dioxus::LaunchBuilder::desktop()
            .with_context(Arc::new(Mutex::new(client)))
            .with_cfg(
                Config::new()
                    .with_window(
                        WindowBuilder::new()
                            .with_transparent(true)
                            .with_decorations(false)
                            .with_parent_window(hwnd.0 as isize)
                            .with_always_on_bottom(true)
                            .with_maximized(true)
                            .with_title(OVERLAY_WINDOW_TITLE),
                    )
                    .with_custom_event_handler(move |event, _window| match event {
                        Event::MainEventsCleared => unsafe {
                            let hotkey_pressed = GetAsyncKeyState(VK_F.0 as i32) < 0
                                && GetAsyncKeyState(VK_SHIFT.0 as i32) < 0;

                            if hotkey_pressed && !HOTKEY_WAS_PRESSED {
                                OVERLAY_VISIBLE = !OVERLAY_VISIBLE;
                            }
                            HOTKEY_WAS_PRESSED = hotkey_pressed;

                            if OVERLAY_VISIBLE {
                                show_overlay(hwnd);
                            } else {
                                hide_overlay(hwnd);
                                restore_game_focus(hwnd);
                            }
                        },
                        _ => {}
                    }),
            )
            .launch(ui::components::app);
    }
}

fn hide_overlay(parent_hwnd: HWND) {
    unsafe {
        let mut current = FindWindowExW(parent_hwnd, HWND::default(), None, None);

        while let Ok(hwnd) = current {
            if hwnd.0.is_null() {
                break;
            }

            let mut title = [0u16; 512];
            GetWindowTextW(hwnd, &mut title);

            if String::from_utf16_lossy(&title).trim_end_matches('\0') == OVERLAY_WINDOW_TITLE {
                // Add transparency and input pass-through flags
                let style = WS_EX_LAYERED.0 as isize
                    | WS_EX_TRANSPARENT.0 as isize
                    | WS_EX_NOACTIVATE.0 as isize
                    | WS_EX_TOOLWINDOW.0 as isize
                    | WS_EX_NOREDIRECTIONBITMAP.0 as isize;

                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
                ShowWindow(hwnd, SW_HIDE);
                EnableWindow(hwnd, false);
                return;
            }

            current = FindWindowExW(parent_hwnd, hwnd, None, None);
        }
    }
}

fn show_overlay(parent_hwnd: HWND) {
    unsafe {
        let mut current = FindWindowExW(parent_hwnd, HWND::default(), None, None);

        while let Ok(hwnd) = current {
            if hwnd.0.is_null() {
                break;
            }

            let mut title = [0u16; 512];
            GetWindowTextW(hwnd, &mut title);

            if String::from_utf16_lossy(&title).trim_end_matches('\0') == OVERLAY_WINDOW_TITLE {
                // Keep only necessary flags for overlay visibility
                let style = WS_EX_LAYERED.0 as isize | WS_EX_NOREDIRECTIONBITMAP.0 as isize;

                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
                ShowWindow(hwnd, SW_SHOW);
                EnableWindow(hwnd, true);
                return;
            }

            current = FindWindowExW(parent_hwnd, hwnd, None, None);
        }
    }
}

fn restore_game_focus(game_hwnd: HWND) {
    unsafe {
        let _ = SetFocus(game_hwnd);
    }
}

fn find_game_window() -> Result<HWND, Box<dyn std::error::Error>> {
    let mut overlay_hwnd = HWND(std::ptr::null_mut());

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows),
            LPARAM(&mut overlay_hwnd as *mut HWND as isize),
        );
    }

    if overlay_hwnd.0 != std::ptr::null_mut() {
        Ok(overlay_hwnd)
    } else {
        Err("Overlay window not found".into())
    }
}

unsafe extern "system" fn enum_windows(window: HWND, param: LPARAM) -> BOOL {
    let mut title: [u16; 512] = [0; 512];
    GetWindowTextW(window, &mut title);

    let window_text = String::from_utf16_lossy(&title)
        .trim_end_matches('\0')
        .to_string();

    if window_text.contains(MAIN_WINDOW_TITLE) {
        *(param.0 as *mut HWND) = window;
        return BOOL(0); // Return FALSE to stop enumeration
    }
    BOOL(1) // Continue enumeration
}
