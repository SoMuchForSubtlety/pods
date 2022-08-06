use adw::traits::BinExt;
use gtk::gdk;
use gtk::glib;
use gtk::glib::closure;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;
use crate::utils;
use crate::view;
use crate::window::Window;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/containers-panel.ui")]
    pub(crate) struct ContainersPanel {
        pub(super) container_list: WeakRef<model::ContainerList>,
        #[template_child]
        pub(super) main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub(super) overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub(super) progress_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub(super) containers_group: TemplateChild<view::ContainersGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainersPanel {
        const NAME: &'static str = "ContainersPanel";
        type Type = super::ContainersPanel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.add_binding_action(
                gdk::Key::N,
                gdk::ModifierType::CONTROL_MASK,
                "containers.create",
                None,
            );
            klass.install_action("containers.create", None, move |widget, _, _| {
                widget.create_container();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContainersPanel {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "container-list",
                    "Container List",
                    "The list of containers",
                    model::ContainerList::static_type(),
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
                "container-list" => obj.set_container_list(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "container-list" => obj.container_list().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let container_list_expr = Self::Type::this_expression("container-list");
            let container_list_len_expr =
                container_list_expr.chain_property::<model::ContainerList>("len");
            let fetching_exprs = &[
                container_list_expr.chain_property::<model::ContainerList>("fetched"),
                container_list_expr.chain_property::<model::ContainerList>("to-fetch"),
            ];

            gtk::ClosureExpression::new::<gtk::Widget, _, _>(
                &[
                    container_list_len_expr,
                    container_list_expr.chain_property::<model::ContainerList>("listing"),
                ],
                closure!(|obj: Self::Type, len: u32, listing: bool| {
                    let imp = obj.imp();
                    if len == 0 && listing {
                        imp.spinner.upcast_ref::<gtk::Widget>().clone()
                    } else {
                        imp.overlay.upcast_ref::<gtk::Widget>().clone()
                    }
                }),
            )
            .bind(&*self.main_stack, "visible-child", Some(obj));

            gtk::ClosureExpression::new::<f64, _, _>(
                fetching_exprs,
                closure!(|_: glib::Object, fetched: u32, to_fetch: u32| {
                    f64::min(1.0, fetched as f64 / to_fetch as f64)
                }),
            )
            .bind(&*self.progress_bar, "fraction", Some(obj));

            gtk::ClosureExpression::new::<bool, _, _>(
                fetching_exprs,
                closure!(|_: glib::Object, fetched: u32, to_fetch: u32| fetched < to_fetch),
            )
            .bind(&*self.progress_revealer, "reveal-child", Some(obj));

            gtk::Revealer::this_expression("child-revealed")
                .chain_closure::<u32>(closure!(|_: glib::Object, revealed: bool| {
                    if revealed {
                        1000_u32
                    } else {
                        0_u32
                    }
                }))
                .bind(
                    &*self.progress_revealer,
                    "transition-duration",
                    Some(&*self.progress_revealer),
                );
        }

        fn dispose(&self, _obj: &Self::Type) {
            self.main_stack.unparent();
        }
    }

    impl WidgetImpl for ContainersPanel {}
}

glib::wrapper! {
    pub(crate) struct ContainersPanel(ObjectSubclass<imp::ContainersPanel>)
        @extends gtk::Widget;
}

impl Default for ContainersPanel {
    fn default() -> Self {
        glib::Object::new(&[]).expect("Failed to create ContainersPanel")
    }
}

impl ContainersPanel {
    pub(crate) fn container_list(&self) -> Option<model::ContainerList> {
        self.imp().container_list.upgrade()
    }

    pub(crate) fn set_container_list(&self, value: &model::ContainerList) {
        if self.container_list().as_ref() == Some(value) {
            return;
        }
        self.imp().container_list.set(Some(value));
        self.notify("container-list");
    }

    fn create_container(&self) {
        let leaflet_overlay = utils::find_leaflet_overlay(self);

        if leaflet_overlay.child().is_none() {
            leaflet_overlay.show_details(&view::ContainerCreationPage::from(
                self.root()
                    .unwrap()
                    .downcast::<Window>()
                    .unwrap()
                    .connection_manager()
                    .client()
                    .as_ref(),
            ));
        }
    }
}
