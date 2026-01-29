use tauri::WebviewWindow;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static LISTENING: AtomicBool = AtomicBool::new(false);

pub fn run_hotkey_listener(window: WebviewWindow) {
    if LISTENING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    let mut pressed_keys: HashSet<String> = HashSet::new();
    let mut hidden = false;

    loop {
        for vk in 0x41..=0x5A {
            if is_key_pressed(vk) {
                let key_name = format!("{}", (vk as u8) as char);
                pressed_keys.insert(key_name);
            }
        }

        if is_key_pressed(0x12) && is_key_pressed(0x20) {
            if hidden {
                let _ = window.show();
                let _ = window.set_focus();
                hidden = false;
            }
        }

        if is_key_pressed(0x1B) {
            let _ = window.hide();
            hidden = true;
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn is_key_pressed(vk_code: u32) -> bool {
    unsafe {
        GetAsyncKeyState(vk_code) & 0x8000 != 0
    }
}

#[link(name = "user32")]
extern "system" {
    fn GetAsyncKeyState(vk_code: u32) -> u16;
}
