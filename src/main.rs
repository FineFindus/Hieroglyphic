mod application;
#[rustfmt::skip]
mod config;
mod classify;
mod widgets;
mod window;

use gettextrs::{LocaleCategory, gettext};
use gtk::{gio, glib};

use self::application::HieroglyphicApplication;
use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

/// If running inside a macOS `.app` bundle, returns the `Contents/Resources/share` path.
#[cfg(target_os = "macos")]
fn macos_bundle_share_dir() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let contents = exe.parent()?.parent()?;
    let share = contents.join("Resources").join("share");
    share.is_dir().then_some(share)
}

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    unsafe {
        // ensure that adwaita is used
        std::env::remove_var("GTK_THEME");
    }

    // On macOS, resolve resource paths relative to the .app bundle if present
    #[cfg(target_os = "macos")]
    let (resources_file, localedir) = match macos_bundle_share_dir() {
        Some(share) => (
            share
                .join("hieroglyphic")
                .join("resources.gresource")
                .to_string_lossy()
                .into_owned(),
            share.join("locale").to_string_lossy().into_owned(),
        ),
        None => (RESOURCES_FILE.to_string(), LOCALEDIR.to_string()),
    };
    #[cfg(not(target_os = "macos"))]
    let (resources_file, localedir) = (RESOURCES_FILE, LOCALEDIR);

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, &localedir)
        .expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("Hieroglyphic"));

    let res = gio::Resource::load(&resources_file).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = HieroglyphicApplication::default();
    app.run()
}
