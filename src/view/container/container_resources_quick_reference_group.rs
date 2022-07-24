use std::cell::Cell;

use adw::subclass::prelude::PreferencesGroupImpl;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::closure;
use gtk::glib::closure_local;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(
        resource = "/com/github/marhkb/Pods/ui/container-resources-quick-reference-group.ui"
    )]
    pub(crate) struct ContainerResourcesQuickReferenceGroup {
        pub(super) container: WeakRef<model::Container>,
        #[template_child]
        pub(super) cpu_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) cpu_progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub(super) memory_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) memory_progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub(super) network_down_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) network_up_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) block_down_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) block_up_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContainerResourcesQuickReferenceGroup {
        const NAME: &'static str = "ContainerResourcesQuickReferenceGroup";
        type Type = super::ContainerResourcesQuickReferenceGroup;
        type ParentType = adw::PreferencesGroup;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContainerResourcesQuickReferenceGroup {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "container",
                    "Container",
                    "The container of this ContainerResourcesQuickReferenceGroup",
                    model::Container::static_type(),
                    glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT,
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
                "container" => {
                    self.container.set(value.get().unwrap());
                }
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

            let stats_expr = Self::Type::this_expression("container")
                .chain_property::<model::Container>("stats");

            stats_expr
                .chain_closure::<String>(closure!(
                    |_: glib::Object, stats: Option<model::BoxedContainerStats>| {
                        gettext!(
                            "{} %",
                            stats
                                .and_then(|stats| stats.CPU.map(|perc| format!("{perc:.1}")))
                                .unwrap_or_else(|| gettext("?")),
                        )
                    }
                ))
                .bind(&*self.cpu_label, "label", Some(obj));

            obj.bind_stats_fraction(
                stats_expr.upcast_ref(),
                |stats| stats.CPU,
                &*self.cpu_progress_bar,
            );

            stats_expr
                .chain_closure::<String>(closure!(
                    |_: glib::Object, stats: Option<model::BoxedContainerStats>| {
                        stats
                            .map(|stats| {
                                gettext!(
                                    "{} / {} ({} %)",
                                    stats
                                        .mem_usage
                                        .map(|usage| String::from(glib::format_size(usage as u64)))
                                        .unwrap_or_else(|| gettext("?")),
                                    stats
                                        .mem_limit
                                        .map(|limit| String::from(glib::format_size(limit as u64)))
                                        .unwrap_or_else(|| gettext("?")),
                                    stats
                                        .mem_perc
                                        .map(|perc| format!("{perc:.1}"))
                                        .unwrap_or_else(|| gettext("?")),
                                )
                            })
                            .unwrap_or_else(|| gettext("?"))
                    }
                ))
                .bind(&*self.memory_label, "label", Some(obj));

            obj.bind_stats_fraction(
                stats_expr.upcast_ref(),
                |stats| stats.mem_perc,
                &*self.memory_progress_bar,
            );

            obj.bind_stats_throughput(
                stats_expr.upcast_ref(),
                |stats| stats.net_input,
                &*self.network_down_label,
            );

            obj.bind_stats_throughput(
                stats_expr.upcast_ref(),
                |stats| stats.net_output,
                &*self.network_up_label,
            );

            obj.bind_stats_throughput(
                stats_expr.upcast_ref(),
                |stats| stats.block_input,
                &*self.block_down_label,
            );

            obj.bind_stats_throughput(
                stats_expr.upcast_ref(),
                |stats| stats.block_output,
                &*self.block_up_label,
            );
        }
    }

    impl WidgetImpl for ContainerResourcesQuickReferenceGroup {}
    impl PreferencesGroupImpl for ContainerResourcesQuickReferenceGroup {}
}

glib::wrapper! {
    pub(crate) struct ContainerResourcesQuickReferenceGroup(ObjectSubclass<imp::ContainerResourcesQuickReferenceGroup>)
        @extends gtk::Widget, adw::PreferencesGroup;
}

impl ContainerResourcesQuickReferenceGroup {
    fn bind_stats_fraction<F>(
        &self,
        stats_expr: &gtk::Expression,
        fraction_op: F,
        progress_bar: &gtk::ProgressBar,
    ) where
        F: Fn(model::BoxedContainerStats) -> Option<f32> + Clone + 'static,
    {
        let fraction_op_clone = fraction_op.clone();
        stats_expr
            .chain_closure::<f64>(closure_local!(|_: glib::Object,
                                                  stats: Option<
                model::BoxedContainerStats,
            >| {
                stats
                    .and_then(|stats| fraction_op_clone(stats).map(|perc| perc as f64 * 0.01))
                    .unwrap_or_default()
            }))
            .bind(progress_bar, "fraction", Some(self));

        let classes = progress_bar.css_classes();
        stats_expr
            .chain_closure::<Vec<String>>(closure_local!(|_: glib::Object,
                                                          stats: Option<
                model::BoxedContainerStats,
            >| {
                classes
                    .iter()
                    .cloned()
                    .chain(stats.and_then(|stats| {
                        fraction_op(stats).and_then(|perc| {
                            if perc >= 80. {
                                Some(glib::GString::from(if perc < 95. {
                                    "progressbar-warning"
                                } else {
                                    "progressbar-error"
                                }))
                            } else {
                                None
                            }
                        })
                    }))
                    .collect::<Vec<_>>()
            }))
            .bind(progress_bar, "css-classes", Some(self));
    }

    fn bind_stats_throughput<F>(
        &self,
        stats_expr: &gtk::Expression,
        throughput_op: F,
        label: &gtk::Label,
    ) where
        F: Fn(model::BoxedContainerStats) -> Option<i64> + 'static,
    {
        let prev_value = Cell::new(-1);

        stats_expr
            .chain_closure::<String>(closure_local!(move |_: Self,
                                                          stats: Option<
                model::BoxedContainerStats,
            >| {
                stats
                    .and_then(|stats| {
                        throughput_op(stats).map(|value| {
                            let s = gettext!(
                                // Translators: For example 5 MB / s.
                                "{} / s",
                                glib::format_size(if prev_value.get() == -1 {
                                    0
                                } else {
                                    (value - prev_value.get()) as u64
                                })
                            );

                            prev_value.set(value);

                            s
                        })
                    })
                    .unwrap_or_else(|| gettext("?"))
            }))
            .bind(label, "label", Some(self));
    }

    pub(crate) fn container(&self) -> Option<model::Container> {
        self.imp().container.upgrade()
    }
}
