use std::path::Path;

use crate::config;
use crate::config::{APP_ID, VERSION};
use adw::prelude::*;
use adw::AboutWindow;
use gettextrs::gettext;
use glib::object::IsA;
use gtk::glib;
use gtk::{Application, License, Window};

const COPYRIGHT: &str = "Copyright © 2024 FineFindus";
const WEBSITE: &str = "https://github.com/FineFindus/Hieroglyphic/";
const ISSUE_TRACKER: &str = "https://github.com/FineFindus/Hieroglyphic/issues/";
const DEVELOPER_NAME: &str = "FineFindus";
const DEVELOPERS: &[&str] = &["Zoey Sheffield", "FineFindus https://github.com/FineFindus"];
const DESIGNERS: &[&str] = &["Tobias Bernard"];
const ARTISTS: &[&str] = &["Tobias Bernard"];

#[derive(Debug)]
pub struct TeXMatchAbout;

impl TeXMatchAbout {
    pub fn show<A: IsA<Application> + AdwApplicationExt, W: IsA<Window> + GtkWindowExt>(
        app: &A,
        window: &W,
    ) {
        let about_window = AboutWindow::builder()
            .application(app)
            .transient_for(window)
            .modal(true)
            .application_icon(APP_ID)
            .application_name(gettext("Hieroglyphic"))
            .developer_name(DEVELOPER_NAME)
            .developers(DEVELOPERS)
            .designers(DESIGNERS)
            .artists(ARTISTS)
            // Translators: This should not be translate, Please enter your credits here instead (format: "Name https://example.com" or "Name <email@example.com>", no quotes)
            .translator_credits(gettext("translator-credits"))
            .license_type(License::Gpl30)
            .version(VERSION)
            .website(WEBSITE)
            .issue_url(ISSUE_TRACKER)
            .copyright(COPYRIGHT)
            .debug_info(Self::debug_info())
            .debug_info_filename("tex_match_debug")
            .build();
        about_window.present();
    }

    ///Returns useful information for debugging the application.
    fn debug_info() -> String {
        let mut information = String::new();

        //information about the app
        information.push_str(&format!("{}: {}\n", config::APP_ID, config::VERSION));
        information.push_str(&format!("Profile: {}\n", config::PROFILE));
        information.push_str(&format!(
            "Backend: {}\n",
            Self::backend().unwrap_or("Failed to get backend")
        ));

        //used OS infos
        information.push_str("OS:\n");
        information.push_str(&format!(
            " - Name: {:?}\n",
            gtk::glib::os_info("NAME").unwrap_or("Failed to get OS NAME".into())
        ));
        information.push_str(&format!(
            " - Version: {:?}\n",
            gtk::glib::os_info("VERSION").unwrap_or("Failed to get VERSION".into())
        ));
        //used libraries version
        information.push_str("Libraries:\n");
        information.push_str(&format!(
            " - GTK: {}.{}.{}\n",
            gtk::major_version(),
            gtk::minor_version(),
            gtk::micro_version()
        ));
        information.push_str(&format!(
            " - Libadwaita: {}.{}.{}\n",
            adw::major_version(),
            adw::minor_version(),
            adw::micro_version()
        ));
        information.push('\n');

        information.push_str(&format!("Sandbox: {}\n", Self::sandbox_info()));
        information.push('\n');

        information
    }

    ///Returns the used display server
    fn backend() -> Option<&'static str> {
        let display = gtk::gdk::Display::default()?;
        //get display backend
        Some(match display.backend() {
            gtk::gdk::Backend::Wayland => "Wayland",
            gtk::gdk::Backend::X11 => "X11",
            gtk::gdk::Backend::Win32 => "Win32",
            gtk::gdk::Backend::MacOS => "MacOS",
            gtk::gdk::Backend::Broadway => "Broadway",
        })
    }

    /// Returns info about the sandbox the app is using.
    ///
    /// If it is running inside the flatpak sandbox, info about it is returned, otherwise only
    /// the information if the `GTK_USE_PORTAL` environment is set to `1`.  
    fn sandbox_info() -> String {
        let mut info = String::new();
        if Path::new("/.flatpak-info").exists() {
            info.push_str("Flatpak Info:\n");
            let info_file = std::fs::read_to_string("/.flatpak-info").unwrap_or_default();
            info_file
                .split('\n')
                .filter_map(|line| line.split_once('='))
                .for_each(|(name, value)| match name {
                    "name" => info.push_str(&format!(" - Name: {}\n", value)),
                    "runtime" => info.push_str(&format!(" - Runtime: {}\n", value)),
                    "runtime-commit" => info.push_str(&format!(" - Runtime commit: {}\n", value)),
                    "arch" => info.push_str(&format!(" - Arch: {}\n", value)),
                    "flatpak-version" => info.push_str(&format!(" - Flatpak Version: {}\n", value)),
                    "devel" => info.push_str(&format!(" - Devel: {}\n", value)),
                    _ => {}
                });
        } else {
            let gtk_portal_env = std::env::var("GTK_USE_PORTAL")
                .map(|v| v == "1")
                .unwrap_or_default();
            info.push_str(&format!(" - GTK_USE_PORTAL: {}\n", gtk_portal_env));
        }

        info
    }
}
