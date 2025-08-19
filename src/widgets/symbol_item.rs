use gio::glib::property::PropertySet;
use gio::glib::variant::ToVariant;
use glib::Object;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::ObjectExt};

use crate::classify;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/symbol-item.ui")]
    #[properties(wrapper_type = super::SymbolItem)]
    pub struct SymbolItem {
        #[property(construct_only, get)]
        pub(super) id: RefCell<String>,
        #[property(construct_only, get)]
        pub(super) icon: RefCell<String>,
        #[property(construct_only, get)]
        pub(super) tex_package_and_mode: RefCell<String>,
        #[property(construct_only, get)]
        pub(super) tex_command: RefCell<String>,
        #[property(construct_only, get)]
        pub(super) typst_command: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SymbolItem {
        const NAME: &'static str = "SymbolItem";
        type ParentType = gtk::Box;
        type Type = super::SymbolItem;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SymbolItem {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for SymbolItem {}
    impl BoxImpl for SymbolItem {}
}

glib::wrapper! {
    pub struct SymbolItem(ObjectSubclass<imp::SymbolItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable, gtk::Actionable;
}

#[gtk::template_callbacks]
impl SymbolItem {
    pub fn new(symbol: classify::Symbol) -> Self {
        Object::builder()
            .property("id", symbol.id())
            .property(
                "icon",
                // icon filenames do not contain ending '='
                format!("{}-symbolic", symbol.id().trim_end_matches('=')),
            )
            .property("tex-command", symbol.command)
            .property(
                "tex-package-and-mode",
                format!("{} ({})", symbol.package, symbol.mode_description()),
            )
            .property(
                "typst-command",
                symbol
                    .typst_command
                    .map(|cmd| format!("Typst: {cmd}"))
                    .unwrap_or_default(),
            )
            .build()
    }

    /// Sets the displayed symbol to the given symbol.
    pub fn set_symbol(&self, symbol: classify::Symbol) {
        let imp = self.imp();
        imp.id.replace(symbol.id().to_string());
        imp.icon
            // icon filenames do not contain ending '='
            .set(format!("{}-symbolic", symbol.id().trim_end_matches('=')));
        imp.tex_command.set(symbol.command.to_string());
        imp.tex_package_and_mode.set(format!(
            "{} ({})",
            symbol.package,
            symbol.mode_description()
        ));
        imp.typst_command.set(
            symbol
                .typst_command
                .map(|cmd| format!("Typst: {cmd}"))
                .unwrap_or_default()
                .to_string(),
        );

        for property_name in ["id", "icon", "command", "package", "mode"] {
            self.notify(property_name);
        }
    }

    #[template_callback]
    pub fn on_copy(&self) {
        self.activate_action("win.copy-symbol", Some(&self.id().to_variant()))
            .expect("Failed to copy symbol");
    }
}
