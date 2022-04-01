use std::cell::RefCell;

use adw::subclass::prelude::{ActionRowImpl, PreferencesRowImpl};
use gtk::glib::{clone, closure, WeakRef};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use once_cell::sync::Lazy;

use crate::{model, utils, view};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/container-row.ui")]
    pub(crate) struct ContainerRow {
        pub(super) container: WeakRef<model::Container>,
        pub(super) handler_id: RefCell<Option<glib::SignalHandlerId>>,
        #[template_child]
        pub(super) stats_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) cpu_bar: TemplateChild<view::CircularProgressBar>,
        #[template_child]
        pub(super) mem_bar: TemplateChild<view::CircularProgressBar>,
        #[template_child]
        pub(super) status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) menu_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainerRow {
        const NAME: &'static str = "ContainerRow";
        type Type = super::ContainerRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("container.show-details", None, move |widget, _, _| {
                widget.show_details();
            });

            klass.install_action("container.start", None, move |widget, _, _| {
                super::super::start(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.stop", None, move |widget, _, _| {
                super::super::stop(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.force-stop", None, move |widget, _, _| {
                super::super::force_stop(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.restart", None, move |widget, _, _| {
                super::super::restart(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.force-restart", None, move |widget, _, _| {
                super::super::force_restart(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.pause", None, move |widget, _, _| {
                super::super::pause(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.resume", None, move |widget, _, _| {
                super::super::resume(widget.upcast_ref(), &widget.container().unwrap());
            });

            klass.install_action("container.rename", None, move |widget, _, _| {
                super::super::rename(widget.upcast_ref(), widget.container());
            });

            klass.install_action("container.commit", None, move |widget, _, _| {
                super::super::commit(widget.upcast_ref(), &widget.container().unwrap());
            });

            klass.install_action("container.delete", None, move |widget, _, _| {
                super::super::delete(widget.upcast_ref(), &widget.container().unwrap());
            });
            klass.install_action("container.force-delete", None, move |widget, _, _| {
                super::super::force_delete(widget.upcast_ref(), &widget.container().unwrap());
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContainerRow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "container",
                    "container",
                    "The Container of this ContainerRow",
                    model::Container::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::EXPLICIT_NOTIFY,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "container" => obj.set_container(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "container" => obj.container().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let container_expr = Self::Type::this_expression("container");
            let stats_expr = container_expr.chain_property::<model::Container>("stats");
            let status_expr = container_expr.chain_property::<model::Container>("status");

            container_expr
                .chain_property::<model::Container>("name")
                .chain_closure::<String>(closure!(|_: glib::Object, name: Option<String>| {
                    utils::escape(&utils::format_option(name))
                }))
                .bind(obj, "title", Some(obj));

            container_expr
                .chain_property::<model::Container>("image-name")
                .chain_closure::<String>(closure!(|_: glib::Object, name: Option<String>| {
                    utils::escape(&utils::format_option(name))
                }))
                .bind(obj, "subtitle", Some(obj));

            status_expr
                .chain_closure::<bool>(closure!(
                    |_: glib::Object, status: model::ContainerStatus| matches!(
                        status,
                        model::ContainerStatus::Running
                    )
                ))
                .bind(&*self.stats_box, "visible", Some(obj));

            stats_expr
                .chain_closure::<f64>(closure!(
                    |_: glib::Object, stats: Option<model::BoxedContainerStats>| {
                        stats
                            .and_then(|stats| stats.CPU.map(|perc| perc as f64 * 0.01))
                            .unwrap_or_default()
                    }
                ))
                .bind(&*self.cpu_bar, "percentage", Some(obj));

            stats_expr
                .chain_closure::<f64>(closure!(
                    |_: glib::Object, stats: Option<model::BoxedContainerStats>| {
                        stats
                            .and_then(|stats| stats.mem_perc.map(|perc| perc as f64 * 0.01))
                            .unwrap_or_default()
                    }
                ))
                .bind(&*self.mem_bar, "percentage", Some(obj));

            status_expr
                .chain_closure::<String>(closure!(
                    |_: glib::Object, status: model::ContainerStatus| status.to_string()
                ))
                .bind(&*self.status_label, "label", Some(obj));

            let css_classes = self.status_label.css_classes();
            status_expr
                .chain_closure::<Vec<String>>(closure!(
                    |_: glib::Object, status: model::ContainerStatus| {
                        css_classes
                            .iter()
                            .cloned()
                            .chain(Some(glib::GString::from(
                                super::super::container_status_css_class(status),
                            )))
                            .collect::<Vec<_>>()
                    }
                ))
                .bind(&*self.status_label, "css-classes", Some(obj));

            container_expr
                .chain_property::<model::Container>("action-ongoing")
                .chain_closure::<String>(closure!(|_: glib::Object, action_ongoing: bool| {
                    if action_ongoing {
                        "ongoing"
                    } else {
                        "menu"
                    }
                }))
                .bind(&*self.menu_stack, "visible-child-name", Some(obj));

            status_expr
                .chain_closure::<Option<gio::MenuModel>>(closure!(
                    |_: Self::Type, status: model::ContainerStatus| {
                        use model::ContainerStatus::*;

                        Some(match status {
                            Running => super::super::running_menu(),
                            Paused => super::super::paused_menu(),
                            Configured | Created | Exited | Dead | Stopped => {
                                super::super::stopped_menu()
                            }
                            _ => return None,
                        })
                    }
                ))
                .bind(&*self.menu_button, "menu-model", Some(obj));
        }
    }

    impl WidgetImpl for ContainerRow {}
    impl ListBoxRowImpl for ContainerRow {}
    impl PreferencesRowImpl for ContainerRow {}
    impl ActionRowImpl for ContainerRow {}
}

glib::wrapper! {
    pub(crate) struct ContainerRow(ObjectSubclass<imp::ContainerRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow;
}

impl From<&model::Container> for ContainerRow {
    fn from(container: &model::Container) -> Self {
        glib::Object::new(&[("container", container)]).expect("Failed to create ContainerRow")
    }
}

impl ContainerRow {
    pub(crate) fn container(&self) -> Option<model::Container> {
        self.imp().container.upgrade()
    }

    fn set_container(&self, value: Option<&model::Container>) {
        let container = self.container();

        if container.as_ref() == value {
            return;
        }

        let imp = self.imp();

        if let Some(container) = container {
            container.disconnect(imp.handler_id.take().unwrap());
        }

        if let Some(container) = value {
            let handler_id = container.connect_notify_local(
                Some("name"),
                clone!(@weak self as obj => move |_, _| {
                    glib::timeout_add_seconds_local_once(
                        1,
                        clone!(@weak obj => move || {
                            let panel = obj
                                .ancestor(view::ContainersPanel::static_type())
                                .unwrap()
                                .downcast::<view::ContainersPanel>()
                                .unwrap();

                            panel.update_search_filter();
                            panel.update_sorter();
                        }),
                    );
                }),
            );
            imp.handler_id.replace(Some(handler_id));
        }

        imp.container.set(value);
        self.notify("container");
    }

    fn show_details(&self) {
        utils::find_leaflet_overlay(self)
            .show_details(&view::ContainerPage::from(&self.container().unwrap()));
    }
}
