use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::closure;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;
use crate::view;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/container-rename-dialog.ui")]
    pub(crate) struct ContainerRenameDialog {
        pub(super) container: WeakRef<model::Container>,
        #[template_child]
        pub(super) button_rename: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) entry_row: TemplateChild<view::RandomNameEntryRow>,
        #[template_child]
        pub(super) error_label_row: TemplateChild<adw::PreferencesRow>,
        #[template_child]
        pub(super) error_label_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) error_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) id_row: TemplateChild<view::PropertyRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainerRenameDialog {
        const NAME: &'static str = "ContainerRenameDialog";
        type Type = super::ContainerRenameDialog;
        type ParentType = gtk::Dialog;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("container.rename", None, |widget, _, _| {
                widget.rename();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContainerRenameDialog {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "container",
                    "container",
                    "The container to rename",
                    model::Container::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "container" => self.container.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "container" => self.container.upgrade().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if let Some(name) = self.container.upgrade().map(|container| container.name()) {
                self.entry_row.set_text(&name);
                self.entry_row.grab_focus();
            }

            obj.action_set_enabled("container.rename", !self.entry_row.text().is_empty());
            self.entry_row
                .connect_changed(clone!(@weak obj => move |entry| {
                    let imp = obj.imp();
                    imp.entry_row.remove_css_class("error");
                    imp.error_label_revealer.set_reveal_child(false);
                    obj.action_set_enabled("container.rename", !entry.text().is_empty());
                }));

            // Just setting 'obj.set_default_widget(Some(&*self.button_rename));' seems to have no
            // effect.
            self.entry_row
                .connect_activate(clone!(@weak obj => move |_| {
                    obj.imp().button_rename.activate();
                }));

            self.error_label_revealer.connect_child_revealed_notify(
                clone!(@weak obj => move |revealer| {
                    if !revealer.reveals_child() {
                        obj.imp().error_label_row.set_visible(false);
                    }
                }),
            );

            Self::Type::this_expression("container")
                .chain_property::<model::Container>("id")
                .chain_closure::<String>(closure!(|_: Self::Type, id: String| {
                    id.chars().take(12).collect::<String>()
                }))
                .bind(&*self.id_row, "value", Some(obj));
        }
    }

    impl WidgetImpl for ContainerRenameDialog {}
    impl WindowImpl for ContainerRenameDialog {}
    impl DialogImpl for ContainerRenameDialog {}
}

glib::wrapper! {
    pub(crate) struct ContainerRenameDialog(ObjectSubclass<imp::ContainerRenameDialog>)
        @extends gtk::Widget, gtk::Window, gtk::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl From<Option<model::Container>> for ContainerRenameDialog {
    fn from(container: Option<model::Container>) -> Self {
        glib::Object::new(&[("container", &container), ("use-header-bar", &1)])
            .expect("Failed to create ContainerRenameDialog")
    }
}

impl ContainerRenameDialog {
    fn rename(&self) {
        let imp = self.imp();

        if let Some(container) = imp.container.upgrade() {
            let new_name = imp.entry_row.text().to_string();
            container.rename(
                new_name,
                clone!(@weak self as obj => move |result| {
                    match result {
                        Ok(_) => obj.response(gtk::ResponseType::Apply),
                        Err(e) => {
                            let imp = obj.imp();
                            imp.entry_row.add_css_class("error");
                            imp.error_label_row.set_visible(true);
                            imp.error_label_revealer.set_reveal_child(true);
                            imp.error_label.set_text(&e.to_string());
                        }
                    }
                }),
            )
        }
    }
}
