use gio::glib::variant::ToVariant;
use glib::Object;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::ObjectExt};

use crate::classify;

mod imp {

    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/symbol-item.ui")]
    #[properties(wrapper_type = super::SymbolItem)]
    pub struct SymbolItem {
        #[property(construct_only, get)]
        id: RefCell<String>,
        #[property(construct_only, get)]
        icon: RefCell<String>,
        #[property(construct_only, get)]
        package: RefCell<String>,
        #[property(construct_only, get)]
        command: RefCell<String>,
        #[property(construct_only, get)]
        mode: RefCell<String>,
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
            .property("command", symbol.command)
            .property("package", symbol.package)
            .property("mode", symbol.mode_description())
            .build()
    }

    #[template_callback]
    pub fn on_copy(&self) {
        self.activate_action("win.copy-symbol", Some(&self.id().to_variant()))
            .expect("Failed to copy symbol");
    }
}
