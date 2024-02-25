#![windows_subsystem = "windows"]
/*
    Dark Mode Switcher for Windows 10 that runs in the system tray.
*/
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;
use std::{env, io, process::Command};
use winreg::RegKey;

const DARK_ICON_DATA: &[u8] = include_bytes!("../assets/dark.ico");
const LIGHT_ICON_DATA: &[u8] = include_bytes!("../assets/light.ico");
const HKCU_SUB_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
const APP_NAME_KEY: &str = "AppsUseLightTheme";
const SYSTEM_NAME_KEY: &str = "SystemUsesLightTheme";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    Dark,
    Light,
    Default,
}

impl ThemeMode {
    /// Detect the current theme mode
    fn detect_mode() -> ThemeMode {
        let reg_key = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        // Open the registry key and get the value
        if let Ok(sub_key) = reg_key.open_subkey(HKCU_SUB_KEY) {
            // Get the value of the registry key
            if let Ok(dword) = sub_key.get_value::<u32, _>(APP_NAME_KEY) {
                // Return the theme mode
                return if dword == 0 {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
            }
        }

        // Return the default theme mode
        ThemeMode::Light
    }

    /// Set the theme mode
    fn set_theme(mode: u32) -> io::Result<()> {
        let hkcu = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (key, _disp) = hkcu.create_subkey(HKCU_SUB_KEY)?;
        key.set_value(APP_NAME_KEY, &mode)?;
        key.set_value(SYSTEM_NAME_KEY, &mode)?;

        Ok(())
    }

    /// Load a theme file from the arguments list.
    fn load_themes(mode: ThemeMode) -> String {
        let args: Vec<_> = env::args().collect();
        if args.len() > 2 {
            match mode {
                ThemeMode::Light => {
                    return args[2].clone();
                }
                ThemeMode::Dark => {
                    return args[1].clone();
                }
                _ => (),
            }
        }

        String::new()
    }

    /// Set the theme based on a file.
    fn set_theme_file(theme: String) {
        println!("Load file: {}", theme);
        Command::new("cmd")
            .args(["/C", "start", &theme])
            .output()
            .expect("failed to load the windows theme file.");
    }
}

// The system tray UI
#[derive(Default, NwgUi)]
pub struct SystemTray {
    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_control(icon: Some(&data.load_icon()), tip: Some("Dark Switch"))]
    #[nwg_events(MousePressLeftUp: [SystemTray::switch_mode], OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item_exit: nwg::MenuItem,
}

impl SystemTray {
    // Show the system tray menu
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.update_tray_icon();
        self.tray_menu.popup(x, y);
    }

    // Exit the application
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    // Switch the theme mode
    fn switch_mode(&self) {
        let mode = ThemeMode::detect_mode();
        let theme = ThemeMode::load_themes(mode);

        if theme == "" {
            match mode {
                ThemeMode::Dark => {
                    let _ = ThemeMode::set_theme(1);
                }
                ThemeMode::Light => {
                    let _ = ThemeMode::set_theme(0);
                }
                _ => {}
            }
        } else {
            ThemeMode::set_theme_file(theme);
        }

        self.update_tray_icon();
    }

    // Function to switch the tray icon based on the ThemeMode
    pub fn update_tray_icon(&self) {
        // Set the tray icon
        let icon = self.load_icon();
        nwg::TrayNotification::set_icon(&self.tray, &icon);
    }

    // Load the tray icon based on the ThemeMode
    fn load_icon(&self) -> nwg::Icon {
        let mode = ThemeMode::detect_mode();
        match mode {
            ThemeMode::Dark => {
                let icon = nwg::Icon::from_bin(LIGHT_ICON_DATA).expect("Failed to load icon");
                return icon;
            }
            ThemeMode::Light => {
                let icon = nwg::Icon::from_bin(DARK_ICON_DATA).expect("Failed to load icon");
                return icon;
            }
            _ => {
                let icon = nwg::Icon::from_bin(LIGHT_ICON_DATA).expect("Failed to load icon");
                return icon;
            }
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let system_tray_ui_app = SystemTray::build_ui(Default::default()).expect("Failed to build UI");
    system_tray_ui_app.update_tray_icon();
    nwg::dispatch_thread_events();
}
