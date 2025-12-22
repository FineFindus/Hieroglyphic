use gtk::glib;
use gtk::subclass::prelude::*;

use crate::classify;

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Default)]
    pub struct SymbolObject {
        pub(super) symbol: OnceCell<classify::Symbol>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SymbolObject {
        const NAME: &'static str = "SymbolObject";
        type Type = super::SymbolObject;
    }

    impl ObjectImpl for SymbolObject {}
}

glib::wrapper! {
    /// GObject wrapper for [`Symbol`]
    pub struct SymbolObject(ObjectSubclass<imp::SymbolObject>);
}

impl SymbolObject {
    pub fn new(symbol: classify::Symbol) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().symbol.set(symbol).unwrap();
        obj
    }

    /// Returns the [`Symbol`] the object is wrapping.
    pub fn symbol(&self) -> &classify::Symbol {
        // Symbol must be set by construction in `new`
        self.imp().symbol.get().unwrap()
    }
}
