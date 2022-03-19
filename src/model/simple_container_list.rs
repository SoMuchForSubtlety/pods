use std::borrow::Borrow;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use indexmap::map::IndexMap;
use once_cell::sync::Lazy;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct SimpleContainerList(pub(super) RefCell<IndexMap<String, model::Container>>);

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleContainerList {
        const NAME: &'static str = "SimpleContainerList";
        type Type = super::SimpleContainerList;
        type ParentType = glib::Object;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for SimpleContainerList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecUInt::new(
                    "len",
                    "Len",
                    "The length of this list",
                    0,
                    std::u32::MAX,
                    0,
                    glib::ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "len" => obj.len().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.connect_items_changed(|self_, _, _, _| self_.notify("len"));
        }
    }

    impl ListModelImpl for SimpleContainerList {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            model::Container::static_type()
        }

        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.0.borrow().len() as u32
        }

        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.0
                .borrow()
                .get_index(position as usize)
                .map(|(_, obj)| obj.upcast_ref())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct SimpleContainerList(ObjectSubclass<imp::SimpleContainerList>)
        @implements gio::ListModel;
}

impl Default for SimpleContainerList {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create SimpleContainerList")
    }
}

impl SimpleContainerList {
    pub(crate) fn add_container(&self, container: model::Container) {
        let (index, old_value) = self
            .imp()
            .0
            .borrow_mut()
            .insert_full(container.id().unwrap().to_owned(), container);
        self.items_changed(index as u32, if old_value.is_some() { 1 } else { 0 }, 1);
    }

    pub(crate) fn remove_container<Q: Borrow<str> + ?Sized>(&self, id: &Q) {
        let mut list = self.imp().0.borrow_mut();
        if let Some((idx, ..)) = list.shift_remove_full(id.borrow()) {
            drop(list);
            self.items_changed(idx as u32, 1, 0);
        }
    }
}

impl SimpleContainerList {
    pub(crate) fn len(&self) -> u32 {
        self.n_items()
    }
}
