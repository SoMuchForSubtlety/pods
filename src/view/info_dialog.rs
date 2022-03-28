use adw::subclass::prelude::{AdwWindowImpl, PreferencesWindowImpl};
use adw::traits::{ExpanderRowExt, PreferencesWindowExt};
use cascade::cascade;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::{utils, view, PODMAN};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/info-dialog.ui")]
    pub(crate) struct InfoDialog {
        #[template_child]
        pub(super) preferences_page: TemplateChild<adw::PreferencesPage>,

        #[template_child]
        pub(super) version_api_version_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) version_built_time_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) version_git_commit_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) version_go_version_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) version_os_arch_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) version_version_row: TemplateChild<view::PropertyRow>,

        #[template_child]
        pub(super) store_config_file_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_container_store_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) store_container_store_paused_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_container_store_running_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_container_store_stopped_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_graph_driver_name_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_graph_options_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub(super) store_graph_options_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) store_graph_root_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_graph_status_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub(super) store_graph_status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) store_image_store_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_run_root_row: TemplateChild<view::PropertyRow>,
        #[template_child]
        pub(super) store_volume_path_row: TemplateChild<view::PropertyRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InfoDialog {
        const NAME: &'static str = "InfoDialog";
        type Type = super::InfoDialog;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for InfoDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.setup();
        }
    }

    impl WidgetImpl for InfoDialog {}
    impl WindowImpl for InfoDialog {}
    impl AdwWindowImpl for InfoDialog {}
    impl PreferencesWindowImpl for InfoDialog {}
}

glib::wrapper! {
    pub(crate) struct InfoDialog(ObjectSubclass<imp::InfoDialog>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for InfoDialog {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create InfoDialog")
    }
}

impl InfoDialog {
    pub(crate) fn setup(&self) {
        utils::do_async(
            PODMAN.info(),
            clone!(@weak self as obj => move |result| match result {
                Ok(info) => {
                    obj.set_search_enabled(true);

                    let imp = obj.imp();

                    imp.preferences_page.set_visible(true);

                    // Version
                    let version = info.version.as_ref();
                    imp.version_api_version_row.set_value(&utils::format_option(
                        version.and_then(|version| version.api_version.as_ref()),
                    ));
                    imp.version_built_time_row
                        .set_value(&utils::format_option(version.and_then(|version| {
                            version.built.and_then(|t| {
                                glib::DateTime::from_unix_local(t).ok().map(|d| {
                                    d.format(
                                        // Translators: This is a date time format (https://valadoc.org/glib-2.0/GLib.DateTime.format.html)
                                        &gettext("%x %X"),
                                    )
                                    .unwrap()
                                })
                            })
                        })));
                    imp.version_git_commit_row
                        .set_value(&utils::format_option(version.and_then(|version| {
                            version.git_commit.as_ref().and_then(|s| {
                                if s.is_empty() {
                                    None
                                } else {
                                    Some(s)
                                }
                            })
                        })));
                    imp.version_go_version_row.set_value(&utils::format_option(
                        version.and_then(|version| version.go_version.as_ref()),
                    ));
                    imp.version_os_arch_row.set_value(&utils::format_option(
                        version.and_then(|version| version.os_arch.as_ref()),
                    ));
                    imp.version_version_row.set_value(&utils::format_option(
                        version.and_then(|version| version.version.as_ref()),
                    ));

                    // Store
                    let store = info.store.as_ref();
                    imp.store_config_file_row.set_value(&utils::format_option(
                        store.and_then(|store| store.config_file.as_ref()),
                    ));
                    imp.store_container_store_label
                        .set_label(&utils::format_option(store.and_then(|store| {
                            store.container_store.as_ref().and_then(|s| {
                                s.number.map(|n| {
                                    // Translators: "{}" is a placeholder for a cardinal numbers.
                                    gettext!("{} Containers", n)
                                })
                            })
                        })));
                    imp.store_container_store_paused_row
                        .set_value(&utils::format_option(store.and_then(|store| {
                            store
                                .container_store
                                .as_ref()
                                .and_then(|s| s.paused.map(|n| n.to_string()))
                        })));
                    imp.store_container_store_running_row
                        .set_value(&utils::format_option(store.and_then(|store| {
                            store
                                .container_store
                                .as_ref()
                                .and_then(|s| s.running.map(|n| n.to_string()))
                        })));
                    imp.store_container_store_stopped_row
                        .set_value(&utils::format_option(store.and_then(|store| {
                            store
                                .container_store
                                .as_ref()
                                .and_then(|s| s.stopped.map(|n| n.to_string()))
                        })));
                    imp.store_graph_driver_name_row
                        .set_value(&utils::format_option(
                            store.and_then(|store| store.graph_driver_name.as_ref()),
                        ));
                    imp.store_graph_options_label
                        .set_label(&utils::format_option(store.and_then(|store| {
                            store.graph_options.as_ref().map(|o| {
                                // Translators: "{}" is a placeholder for a cardinal number.
                                gettext!("{} Options", o.len())
                            })
                        })));
                    if let Some(graph_options) =
                        store.and_then(|store| store.graph_options.as_ref())
                    {
                        graph_options.iter().for_each(|(k, v)| {
                            imp.store_graph_options_row.add_row(&cascade! {
                                view::PropertyRow::default();
                                ..set_key(k);
                                ..set_value(&v.to_string());
                            });
                        });
                    }
                    imp.store_graph_root_row.set_value(&utils::format_option(
                        info.store
                            .as_ref()
                            .and_then(|store| store.graph_root.as_ref()),
                    ));
                    imp.store_graph_status_label
                        .set_label(&utils::format_option(store.and_then(|store| {
                            store.graph_status.as_ref().map(|s| {
                                // Translators: "{}" is placeholders for a cardinal number.
                                gettext!("{} States", s.len())
                            })
                        })));
                    if let Some(graph_status) = info
                        .store
                        .as_ref()
                        .and_then(|store| store.graph_status.as_ref())
                    {
                        graph_status.iter().for_each(|(k, v)| {
                            imp.store_graph_status_row.add_row(&cascade! {
                                view::PropertyRow::default();
                                ..set_key(k);
                                ..set_value(v);
                            });
                        });
                    }
                    imp.store_image_store_row
                        .set_value(&utils::format_option(store.and_then(|store| {
                            store.image_store.as_ref().and_then(|s| s.number).map(|n| {
                                // Translators: "{}" is placeholders for a cardinal number.
                                gettext!("{} Images", n)
                            })
                        })));
                    imp.store_run_root_row.set_value(&utils::format_option(
                        store.and_then(|store| store.run_root.as_ref()),
                    ));
                    imp.store_volume_path_row.set_value(&utils::format_option(
                        store.and_then(|store| store.volume_path.as_ref()),
                    ));
                    imp.store_volume_path_row.set_value(&utils::format_option(
                        store.and_then(|store| store.volume_path.as_ref()),
                    ));
                }
                Err(e) => {
                    log::error!("Failed to retrieve host info: {e}");
                    obj.add_toast(
                        &adw::Toast::builder()
                            .title(
                                // Translators: The placeholder "{}" is for the error message.
                                &gettext!("Error: {}", e)
                            )
                            .priority(adw::ToastPriority::High)
                            .build(),
                    );
                }}
            ),
        );
    }
}
