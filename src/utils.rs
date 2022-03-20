use std::collections::BTreeSet;
use std::marker::PhantomData;
use std::ops::Deref;

use futures::{Future, Stream, StreamExt};
use gettextrs::gettext;
use gtk::prelude::{Cast, ListModelExt, StaticType};
use gtk::traits::WidgetExt;
use gtk::{gio, glib};
use paste::paste;

use crate::{view, RUNTIME};

macro_rules! boxed_type {
    ($name:ident, $type:ty) => {
        paste! {
            #[derive(Clone, Debug, PartialEq, glib::Boxed)]
            #[boxed_type(name = "" $name "")]
            pub(crate) struct $name(pub(crate) $type);

            impl Deref for $name {
                type Target = $type;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    };
}

boxed_type!(BoxedStringVec, Vec<String>);
boxed_type!(BoxedStringBTreeSet, BTreeSet<String>);

pub(crate) fn find_leaflet_overview<W: glib::IsA<gtk::Widget>>(widget: &W) -> view::LeafletOverlay {
    let leaflet = widget
        .ancestor(adw::Leaflet::static_type())
        .unwrap()
        .downcast::<adw::Leaflet>()
        .unwrap();

    leaflet
        .child_by_name("overlay")
        .unwrap()
        .downcast::<view::LeafletOverlay>()
        .unwrap()
}

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

pub(crate) fn format_option<'a, T>(option: Option<T>) -> String
where
    T: AsRef<str> + 'a,
{
    option.map(|t| String::from(t.as_ref())).unwrap_or_else(||
            // Translators: This string will be shown when a property of an entity like an image is null.
            gettext("<none>"))
}

pub(crate) fn format_iter<'a, I, T: ?Sized>(iter: I, sep: &str) -> String
where
    I: IntoIterator<Item = &'a T>,
    T: AsRef<str> + 'a,
{
    format_option(format_iter_or_none(iter, sep))
}

pub(crate) fn format_iter_or_none<'a, I, T: ?Sized + 'a>(iter: I, sep: &str) -> Option<String>
where
    I: IntoIterator<Item = &'a T>,
    T: AsRef<str> + 'a,
{
    let mut iter = iter.into_iter();
    iter.next().map(|first| {
        Some(first)
            .into_iter()
            .chain(iter)
            .map(|some| some.as_ref())
            .collect::<Vec<_>>()
            .join(sep)
    })
}

// Function from https://gitlab.gnome.org/GNOME/fractal/-/blob/fractal-next/src/utils.rs
pub(crate) fn do_async<R, Fut, F>(tokio_fut: Fut, glib_closure: F)
where
    R: Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    F: FnOnce(R) + 'static,
{
    let handle = RUNTIME.spawn(tokio_fut);

    glib::MainContext::default().spawn_local_with_priority(Default::default(), async move {
        glib_closure(handle.await.unwrap());
    });
}

pub(crate) fn run_stream<S, I, F>(mut stream: S, glib_closure: F)
where
    S: Stream<Item = I> + Send + Unpin + 'static,
    I: Send + 'static,
    F: Fn(I) -> glib::Continue + 'static,
{
    let (sender, receiver) = glib::MainContext::sync_channel::<I>(Default::default(), 5);

    receiver.attach(None, glib_closure);

    RUNTIME.spawn(async move {
        while let Some(item) = stream.next().await {
            if sender.send(item).is_err() {
                break;
            }
        }
    });
}

pub(crate) trait ToTypedListModel {
    fn to_typed_list_model<T>(self) -> TypedListModel<Self, T>
    where
        Self: Sized;
}

impl<M: glib::IsA<gio::ListModel>> ToTypedListModel for M {
    fn to_typed_list_model<T>(self) -> TypedListModel<Self, T>
    where
        Self: Sized,
    {
        TypedListModel::from(self)
    }
}

#[derive(Clone)]
pub(crate) struct TypedListModel<M, T> {
    model: M,
    _phantom: PhantomData<T>,
}

impl<M, T> From<M> for TypedListModel<M, T> {
    fn from(model: M) -> Self {
        Self {
            model,
            _phantom: PhantomData,
        }
    }
}

impl<M, T> TypedListModel<M, T>
where
    M: glib::IsA<gio::ListModel>,
    T: Clone,
{
    pub(crate) fn iter(&self) -> TypedListModelIter<M, T> {
        self.to_owned().into()
    }
}

pub(crate) struct TypedListModelIter<M, T> {
    typed_list_store: TypedListModel<M, T>,
    index: u32,
}

impl<M, T> From<TypedListModel<M, T>> for TypedListModelIter<M, T> {
    fn from(typed_list_store: TypedListModel<M, T>) -> Self {
        TypedListModelIter {
            typed_list_store,
            index: 0,
        }
    }
}

impl<M: glib::IsA<gio::ListModel>, T: glib::IsA<glib::Object>> Iterator
    for TypedListModelIter<M, T>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self
            .typed_list_store
            .model
            .item(self.index)
            .and_then(|o| o.downcast::<T>().ok());
        self.index += 1;
        t
    }
}

impl<M, T> IntoIterator for TypedListModel<M, T>
where
    M: glib::IsA<gio::ListModel>,
    T: glib::IsA<glib::Object>,
{
    type Item = T;
    type IntoIter = TypedListModelIter<M, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into()
    }
}
