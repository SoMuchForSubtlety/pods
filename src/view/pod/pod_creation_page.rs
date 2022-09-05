use adw::subclass::prelude::*;
use adw::traits::BinExt;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::model;
use crate::podman;
use crate::utils;
use crate::view;

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/github/marhkb/Pods/ui/pod-creation-page.ui")]
    pub(crate) struct PodCreationPage {
        pub(super) client: WeakRef<model::Client>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) preferences_page: TemplateChild<adw::PreferencesPage>,
        #[template_child]
        pub(super) name_entry_row: TemplateChild<view::RandomNameEntryRow>,
        #[template_child]
        pub(super) hostname_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub(super) create_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) pod_details_page_bin: TemplateChild<adw::Bin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PodCreationPage {
        const NAME: &'static str = "PodCreationPage";
        type Type = super::PodCreationPage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.install_action("pod.create", None, |widget, _, _| {
                widget.create();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PodCreationPage {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::new(
                    "client",
                    "Client",
                    "The client of this PodCreationPage",
                    model::Client::static_type(),
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
                "client" => self.client.set(value.get().unwrap()),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "client" => obj.client().to_value(),
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            self.name_entry_row
                .connect_text_notify(clone!(@weak obj => move |_| obj.on_name_changed()));
        }

        fn dispose(&self, obj: &Self::Type) {
            let mut child = obj.first_child();
            while let Some(child_) = child {
                child = child_.next_sibling();
                child_.unparent();
            }
        }
    }

    impl WidgetImpl for PodCreationPage {
        fn root(&self, widget: &Self::Type) {
            self.parent_root(widget);

            glib::idle_add_local(
                clone!(@weak widget => @default-return glib::Continue(false), move || {
                    widget.imp().name_entry_row.grab_focus();
                    glib::Continue(false)
                }),
            );
            utils::root(widget).set_default_widget(Some(&*self.create_button));
        }

        fn unroot(&self, widget: &Self::Type) {
            utils::root(widget).set_default_widget(gtk::Widget::NONE);
            self.parent_unroot(widget)
        }
    }
}

glib::wrapper! {
    pub(crate) struct PodCreationPage(ObjectSubclass<imp::PodCreationPage>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl From<Option<&model::Client>> for PodCreationPage {
    fn from(client: Option<&model::Client>) -> Self {
        glib::Object::new(&[("client", &client)]).expect("Failed to create PodCreationPage")
    }
}

impl PodCreationPage {
    fn client(&self) -> Option<model::Client> {
        self.imp().client.upgrade()
    }

    fn on_name_changed(&self) {
        self.action_set_enabled("pod.create", self.imp().name_entry_row.text().len() > 0);
    }

    fn create(&self) {
        self.action_set_enabled("pod.create", false);

        let imp = self.imp();
        imp.preferences_page.set_sensitive(false);

        let opts = podman::opts::PodCreateOpts::builder()
            .name(imp.name_entry_row.text().as_str())
            .hostname(imp.hostname_entry_row.text().as_str())
            .build();

        utils::do_async(
            {
                let podman = self.client().unwrap().podman().clone();
                async move { podman.pods().create(&opts).await }
            },
            clone!(@weak self as obj => move |result| {
                match result.map(|pod| pod.id().to_string()) {
                    Ok(id) => {
                        let client = obj.client().unwrap();
                        match client.pod_list().get_pod(&id) {
                            Some(pod) => obj.switch_to_pod(&pod),
                            None => {
                                client.pod_list().connect_pod_added(
                                    clone!(@weak obj, @strong id => move |_, pod| {
                                        if pod.id() == id.as_str() {
                                            obj.switch_to_pod(pod);
                                        }
                                    }),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error while creating pod: {}", e);
                        utils::show_error_toast(
                            &obj,
                            "Error while creating pod",
                            &e.to_string()
                        );

                        obj.action_set_enabled("pod.create", true);
                        obj.imp().preferences_page.set_sensitive(true);
                    }
                }
            }),
        );
    }

    fn switch_to_pod(&self, pod: &model::Pod) {
        let imp = self.imp();
        imp.pod_details_page_bin
            .set_child(Some(&view::PodDetailsPage::from(pod)));
        imp.stack.set_visible_child(&*imp.pod_details_page_bin);
    }
}
