#![windows_subsystem = "windows"]
/*
    Dark Mode Switcher for Windows 10 that runs in the system tray.
*/
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;
use powershell_script::PsScriptBuilder;
use winreg::RegKey;

const DARK_ICON_DATA: &[u8] = include_bytes!("../assets/dark.ico");
const LIGHT_ICON_DATA: &[u8] = include_bytes!("../assets/light.ico");

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    Dark,
    Light,
    Default,
}

impl ThemeMode {
    /// Detect the current theme mode
    fn detect_mode() -> ThemeMode {
        let key = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
        let value = "AppsUseLightTheme";
        let reg_key = RegKey::predef(winreg::enums::HKEY_CURRENT_USER);

        // Open the registry key and get the value
        if let Ok(sub_key) = reg_key.open_subkey(key) {
            // Get the value of the registry key
            if let Ok(dword) = sub_key.get_value::<u32, _>(value) {
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
        self.tray_menu.popup(x, y);
    }

    // Exit the application
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    // Switch the theme mode
    fn switch_mode(&self) {
        let mode = ThemeMode::detect_mode();

        match mode {
            ThemeMode::Dark => self.switch_to_light(),
            ThemeMode::Light => self.switch_to_dark(),
            _ => {}
        }

        self.update_tray_icon();
    }

    // Switch to dark mode
    fn switch_to_dark(&self) {
        let ps = PsScriptBuilder::new()
            .no_profile(true)
            .non_interactive(true)
            .hidden(false)
            .print_commands(false)
            .build();

        let _output = ps.run(r#"New-ItemProperty -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize -Name SystemUsesLightTheme -Value 0 -Type Dword -Force; New-ItemProperty -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize -Name AppsUseLightTheme -Value 0 -Type Dword -Force"#).unwrap();
    }

    // Switch to light mode
    fn switch_to_light(&self) {
        let ps = PsScriptBuilder::new()
            .no_profile(true)
            .non_interactive(true)
            .hidden(false)
            .print_commands(false)
            .build();

        let _output = ps.run(r#"New-ItemProperty -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize -Name SystemUsesLightTheme -Value 1 -Type Dword -Force; New-ItemProperty -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize -Name AppsUseLightTheme -Value 1 -Type Dword -Force"#).unwrap();
    }

    // Function to switch the tray icon based on the ThemeMode
    pub fn update_tray_icon(&self) {
        // Set the tray icon
        let icon = self.load_icon();
        nwg::TrayNotification::set_icon(&self.tray, &icon);
    }

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
