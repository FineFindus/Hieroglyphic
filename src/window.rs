use std::time::Instant;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::subclass::prelude::*;

use crate::application::HieroglyphicApplication;
use crate::widgets::{BoxedStrokes, SymbolItem};
use crate::{classify, config};

// GTK is single-threaded
thread_local! {
    static SETTINGS: gio::Settings = gio::Settings::new(config::APP_ID);
}

mod imp {
    use std::{
        cell::{OnceCell, RefCell},
        sync::mpsc::Sender,
    };

    use adw::subclass::application_window::AdwApplicationWindowImpl;

    use crate::{
        classify::Symbol,
        config,
        widgets::{self, IndicatorButton},
    };

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/window.ui")]
    #[properties(wrapper_type = super::HieroglyphicWindow)]
    pub struct HieroglyphicWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub drawing_area: TemplateChild<widgets::DrawingArea>,
        #[template_child]
        pub symbol_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub indicator_button: TemplateChild<IndicatorButton>,
        #[template_child]
        pub preview_symbol: TemplateChild<SymbolItem>,
        #[property(get, set)]
        pub stack_page: RefCell<String>,
        pub symbols: OnceCell<gio::ListStore>,
        pub symbol_strokes: RefCell<Option<Vec<classify::Stroke>>>,
        pub classifier: OnceCell<Sender<Vec<classify::Stroke>>>,
        pub symbol_filter: RefCell<Vec<&'static str>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HieroglyphicWindow {
        const NAME: &'static str = "HieroglyphicWindow";
        type Type = super::HieroglyphicWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("win.show-contribution-dialog", None, move |win, _, _| {
                let builder = gtk::Builder::from_resource(
                    "/io/github/finefindus/Hieroglyphic/ui/contribution-dialog.ui",
                );
                let switch: adw::SwitchRow = builder.object("switch_row").unwrap();
                SETTINGS.with(|settings| {
                    settings.bind("contribute-data", &switch, "active").build();
                    // only show nudge once, i.e. hide it after clicking the button
                    settings
                        .set_boolean("show-contribution-nudge", false)
                        .expect("Failed to set `show-contribution-nudge`");
                });
                let dialog: adw::Dialog = builder.object("contribution_dialog").unwrap();
                dialog.present(Some(win));
            });

            klass.install_action("win.clear", None, move |win, _, _| {
                win.imp().drawing_area.clear();
            });

            klass.install_action(
                "win.copy-symbol",
                Some(glib::VariantTy::STRING),
                move |win, _, var| {
                    let Some(symbol) = var
                        .and_then(|v| v.get::<String>())
                        .and_then(|id| Symbol::from_id(&id))
                    else {
                        return;
                    };

                    win.copy_symbol(&SymbolItem::new(symbol));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for HieroglyphicWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if config::PROFILE == "Devel" {
                obj.add_css_class("devel");
                SETTINGS.with(|settings| {
                    settings
                        .set_boolean("show-contribution-nudge", true)
                        .expect("Failed to set `show-contribution-nudge`");
                });
            }

            tracing::debug!("Loaded {} symbols", classify::SYMBOL_COUNT);

            let settings = SETTINGS.with(|s| s.clone());
            settings
                .bind(
                    "show-contribution-nudge",
                    &*self.indicator_button,
                    "show-indicator",
                )
                .build();

            obj.setup_symbol_list();
            obj.setup_classifier();
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for HieroglyphicWindow {}
    impl WindowImpl for HieroglyphicWindow {}

    impl ApplicationWindowImpl for HieroglyphicWindow {}
    impl AdwApplicationWindowImpl for HieroglyphicWindow {}
}

glib::wrapper! {
    pub struct HieroglyphicWindow(ObjectSubclass<imp::HieroglyphicWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

#[gtk::template_callbacks]
impl HieroglyphicWindow {
    pub fn new(app: &HieroglyphicApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    /// Shows a basic toast with the given text.
    fn show_toast(&self, text: impl AsRef<str>) {
        let toast = adw::Toast::builder()
            .title(text.as_ref())
            .use_markup(false)
            .build();
        // dismiss and replace the previous toasts if they exists
        // this ensures that the user only sees the most recent copy toast
        self.imp().toast_overlay.dismiss_all();
        self.imp().toast_overlay.add_toast(toast);
    }

    fn setup_symbol_list(&self) {
        let model = gio::ListStore::new::<gtk::StringObject>();

        self.imp()
            .symbols
            .set(model.clone())
            .expect("Failed to set symbol model");

        let selection_model = gtk::NoSelection::new(Some(model));
        self.imp()
            .symbol_list
            .bind_model(Some(&selection_model), move |obj| {
                let symbol_object = obj
                    .downcast_ref::<gtk::StringObject>()
                    .expect("Object should be of type `StringObject`");
                let symbol_item = SymbolItem::new(
                    classify::Symbol::from_id(&symbol_object.string())
                        .expect("`symbol_object` should be a valid symbol id"),
                );
                symbol_item.upcast()
            });
    }

    fn setup_classifier(&self) {
        let imp = self.imp();
        let (req_tx, req_rx) = std::sync::mpsc::channel();
        let (res_tx, res_rx) = async_channel::bounded(1);
        imp.classifier.set(req_tx).expect("Failed to set tx");

        gio::spawn_blocking(move || {
            tracing::info!("Classifier thread started");
            let classifier = classify::Classifier::new().expect("Failed to setup classifier");

            loop {
                let Some(strokes) = req_rx.iter().next() else {
                    // channel has hung up, cleanly exit
                    tracing::info!("Exiting classifier thread");
                    return;
                };

                if strokes.is_empty() {
                    tracing::warn!("Skipping classification on empty strokes");
                    continue;
                }

                let classifications: Option<Vec<&'static str>> = 'classify: {
                    let start = Instant::now();
                    let Some(results) = classifier.classify(strokes) else {
                        tracing::warn!("Classifier returned None");
                        break 'classify None;
                    };
                    tracing::info!(
                        "Classification complete in {}ms",
                        start.elapsed().as_millis()
                    );
                    Some(results)
                };

                res_tx
                    .send_blocking(classifications)
                    .expect("Failed to send classifications");
            }
        });

        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                let imp = window.imp();
                tracing::debug!("Listening for classifications");
                while let Ok(Some(mut classifications)) = res_rx.recv().await {
                    window.set_stack_page("symbols");
                    let mut symbols = imp
                        .symbols
                        .get()
                        .cloned()
                        .expect("`symbols` should be initialized in `setup_symbol_list`");

                    let filter_symbols = imp.symbol_filter.borrow();
                    if !filter_symbols.is_empty() {
                        // if `--show-only` symbols are set, show only those symbols
                        // this is intended, for development/debug to directly improve new/less recognized
                        // symbols
                        classifications = filter_symbols.clone();
                    }

                    // use the first symbol as the symbol displayed in the bottom bar in
                    // bottom-sheet mode
                    if let Some(symbol) = classifications
                        .first()
                        .and_then(|id| classify::Symbol::from_id(id))
                    {
                        imp.preview_symbol.set_symbol(symbol);
                    }

                    symbols.remove_all();
                    // switching out all 1k symbols takes too long, so only display the first 25
                    symbols.extend(
                        classifications
                            .into_iter()
                            .take(25)
                            .map(&gtk::StringObject::new),
                    );
                    // scroll to top after updating symbols, so that the most likely symbols are
                    // visible first
                    imp.symbol_list
                        .adjustment()
                        .expect("Failed to get symbol list adjustment")
                        .set_value(0.0);
                }
            }
        ));
    }

    /// Classify the given strokes.
    #[template_callback]
    fn classify(&self, BoxedStrokes(strokes): BoxedStrokes) {
        // we clone the strokes to the window, so we can upload them later on
        self.imp().symbol_strokes.replace(Some(strokes.clone()));
        self.imp()
            .classifier
            .get()
            .unwrap()
            .send(strokes)
            .expect("Failed to send strokes");
    }

    #[template_callback]
    fn on_item_activated(&self, row: Option<&gtk::ListBoxRow>) {
        let binding = row.and_then(|row| row.child());
        let Some(symbol) = binding.and_downcast_ref::<SymbolItem>() else {
            return;
        };

        self.copy_symbol(symbol);
    }

    #[template_callback]
    pub fn copy_symbol(&self, symbol: &SymbolItem) {
        let command = symbol.command();
        self.clipboard().set_text(&command);
        tracing::debug!("Selected: {} ({})", &command, symbol.id());
        self.show_toast(gettext("Copied “{}”").replace("{}", &command));

        if let Some(strokes) = self.imp().symbol_strokes.take() {
            self.try_upload_data(symbol.id(), strokes);
        }
    }

    fn try_upload_data(&self, label: String, strokes: Vec<classify::Stroke>) {
        // skip uploads always on debug mode, to avoid accidental uploads
        if SETTINGS.with(|s| !s.boolean("contribute-data")) || config::PROFILE == "Devel" {
            tracing::debug!("Skipping data upload: user has not opted into data contribution");
            return;
        }

        // skip uploading the data if the user is on a metered network connection
        // see https://gitlab.gnome.org/GNOME/Initiatives/-/issues/42
        let network_monitor = gio::NetworkMonitor::default();
        if network_monitor.is_network_metered() {
            tracing::debug!("Skipping data upload: network is metered");
            return;
        }

        // skip uploading data whilst the user has power saving enabled
        // see https://gitlab.gnome.org/GNOME/Initiatives/-/issues/43
        let power_monitor = gio::PowerProfileMonitor::get_default();
        if power_monitor.is_power_saver_enabled() {
            tracing::debug!("Skipping data upload: power saver is active");
            return;
        }

        tracing::info!("Uploading strokes...");
        // spawn a new thread to avoid blocking the UI thread while uploading
        std::thread::spawn(move || {
            match ureq::post(&format!(
                "https://hieroglyphic-server-6g7a.shuttle.app/v1/upload/{}",
                label
            ))
            .send_json(strokes)
            {
                Ok(_) => {
                    tracing::info!("Successfully uploaded data");
                }
                Err(err) => {
                    tracing::warn!("Failed to upload strokes: {}", err);
                }
            }
        });
    }
}
