use adw::prelude::AdwDialogExt;
use tracing::{debug, info};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use crate::config::{self};
use crate::widgets::about;
use crate::window::HieroglyphicWindow;

mod imp {
    use super::*;
    use adw::subclass::application::AdwApplicationImpl;
    use glib::WeakRef;
    use itertools::Itertools;
    use std::cell::OnceCell;

    #[derive(Debug, Default)]
    pub struct HieroglyphicApplication {
        pub window: OnceCell<WeakRef<HieroglyphicWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HieroglyphicApplication {
        const NAME: &'static str = "HieroglyphicApplication";
        type Type = super::HieroglyphicApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for HieroglyphicApplication {}

    impl ApplicationImpl for HieroglyphicApplication {
        fn activate(&self) {
            debug!("Hieroglyphic<HieroglyphicApplication>::activate");
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = HieroglyphicWindow::new(&app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("Hieroglyphic<HieroglyphicApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(config::APP_ID);

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
        }

        fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> glib::ExitCode {
            if command_line
                .arguments()
                .get(1)
                .is_some_and(|arg| arg != "--show-only")
            {
                return glib::ExitCode::FAILURE;
            }

            self.activate();
            let window = self.window.get().unwrap().upgrade().unwrap();
            window.imp().symbol_filter.replace(
                command_line
                    .arguments()
                    .into_iter()
                    .skip(2)
                    .filter_map(|arg| arg.into_string().ok())
                    .collect_vec(),
            );

            glib::ExitCode::SUCCESS
        }
    }

    impl GtkApplicationImpl for HieroglyphicApplication {}
    impl AdwApplicationImpl for HieroglyphicApplication {}
}

glib::wrapper! {
    pub struct HieroglyphicApplication(ObjectSubclass<imp::HieroglyphicApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl HieroglyphicApplication {
    fn main_window(&self) -> HieroglyphicWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                app.main_window().close();
                app.quit();
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();
        self.add_action_entries([action_quit, action_about]);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
        self.set_accels_for_action("win.clear", &["<Control>n", "Delete"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/io/github/finefindus/Hieroglyphic/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        about::window().present(Some(&self.main_window()));
    }

    pub fn run(&self) -> glib::ExitCode {
        info!("Hieroglyphic ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        ApplicationExtManual::run(self)
    }
}

impl Default for HieroglyphicApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .property("resource-base-path", "/io/github/finefindus/Hieroglyphic/")
            .build()
    }
}
