use dioxus::{desktop::{tao::platform::windows::WindowBuilderExtWindows, Config, WindowBuilder}, prelude::*};
use dioxus::desktop::tao::event::Event;
use tracing::info;
use windows::Win32::{Foundation::{BOOL, HWND, LPARAM}, UI::{Input::KeyboardAndMouse::{EnableWindow, SetFocus}, WindowsAndMessaging::{EnumWindows, FindWindowExW, GetWindowLongPtrW, GetWindowTextW, PostMessageW, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SWP_NOMOVE, SWP_NOSIZE, WM_KEYDOWN, WM_KEYUP, WS_DISABLED, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP}}};

const MAIN_WINDOW_TITLE : &str = "Visual Studio Code";
const OVERLAY_WINDOW_TITLE: &str = "Warhorse Game Overlay";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    if let Ok(hwnd) = find_game_window() {
        info!("Game window found: {:?}", hwnd);
        dioxus::LaunchBuilder::desktop()
            .with_cfg(Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_transparent(true)
                        .with_decorations(false)
                        .with_parent_window(hwnd.0 as isize)
                        .with_always_on_bottom(true)
                        .with_maximized(true)
                        .with_title(OVERLAY_WINDOW_TITLE)
                )
                .with_custom_event_handler(move |event, _window| {
                    match event {
                        Event::MainEventsCleared => {
                            disable_overlay_window_interaction(hwnd);
                            restore_game_focus(hwnd);
                        }
                        _ => {},
                    }
                })
            )
            .launch(app);
    }

    Ok(())
}

fn app() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: "/assets/main.css" }
        div {
            "Game Overlay!"
        }
    }
}

fn disable_overlay_window_interaction(parent_hwnd: HWND) -> bool {
    unsafe {
        let mut current = FindWindowExW(parent_hwnd, HWND::default(), None, None);
        
        while let Ok(hwnd) = current {
            if hwnd.0.is_null() { break; }

            let mut title = [0u16; 512];
            GetWindowTextW(hwnd, &mut title);

            if String::from_utf16_lossy(&title).trim_end_matches('\0') == OVERLAY_WINDOW_TITLE {
                let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, 
                    ex_style | WS_EX_LAYERED.0 as isize 
                    | WS_EX_TRANSPARENT.0 as isize 
                    | WS_EX_NOACTIVATE.0 as isize
                    | WS_EX_NOREDIRECTIONBITMAP.0 as isize
                    | WS_EX_TOOLWINDOW.0 as isize
                );

                let _= EnableWindow(hwnd, false);

                return true;
            }

            current = FindWindowExW(parent_hwnd, hwnd, None, None);
        }
        false
    }
}

fn restore_game_focus(game_hwnd: HWND) {
    unsafe {
        let _= SetFocus(game_hwnd);
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
        return BOOL(0);  // Return FALSE to stop enumeration
    }
    BOOL(1)  // Continue enumeration
}
