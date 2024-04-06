use glib::Object;
use gtk::subclass::prelude::*;
use gtk::{glib, prelude::ObjectExt};

mod imp {

    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/symbol-item.ui")]
    #[properties(wrapper_type = super::SymbolItem)]
    pub struct SymbolItem {
        #[property(get, set)]
        pub(super) id: RefCell<String>,
        #[property(get, set)]
        pub(super) icon: RefCell<String>,
        #[property(get, set)]
        pub(super) package: RefCell<String>,
        #[property(get, set)]
        pub(super) command: RefCell<String>,
        #[property(get, set)]
        pub(super) mode: RefCell<String>,
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
        fn constructed(&self) {
            self.parent_constructed();
        }

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
    pub fn new(symbol: detexify::Symbol) -> Self {
        Object::builder()
            .property("id", symbol.id())
            .property("icon", &format!("{}-symbolic", symbol.id()))
            .property("command", symbol.command)
            .property("package", symbol.package)
            .property(
                "mode",
                match (symbol.math_mode, symbol.text_mode) {
                    (true, true) => "mathmode & textmode",
                    (false, true) => "textmode",
                    (true, false) => "mathmode",
                    (false, false) => "",
                },
            )
            .build()
    }
}
