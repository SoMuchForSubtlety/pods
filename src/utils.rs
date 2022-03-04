use std::collections::BTreeSet;
use std::ops::Deref;

use futures::{Future, Stream, StreamExt};
use gettextrs::gettext;
use gtk::glib;
use paste::paste;

use crate::RUNTIME;

macro_rules! boxed_type {
    ($name:ident, $type:ty) => {
        paste! {
            #[derive(Clone, Debug, PartialEq, glib::Boxed)]
            #[boxed_type(name = "" $name "")]
            pub struct $name(pub $type);

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

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

pub fn format_option<'a, T>(option: Option<T>) -> String
where
    T: AsRef<str> + 'a,
{
    option
        .map(|t| String::from(t.as_ref()))
        .unwrap_or_else(|| gettext("<none>"))
}

pub fn format_iter<'a, I, T: ?Sized>(iter: I, sep: &str) -> String
where
    I: IntoIterator<Item = &'a T>,
    T: AsRef<str> + 'a,
{
    format_option(format_iter_or_none(iter, sep))
}

pub fn format_iter_or_none<'a, I, T: ?Sized + 'a>(iter: I, sep: &str) -> Option<String>
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
pub fn do_async<
    R: Send + 'static,
    F1: Future<Output = R> + Send + 'static,
    F2: Future<Output = ()> + 'static,
    FN: FnOnce(R) -> F2 + 'static,
>(
    priority: glib::source::Priority,
    tokio_fut: F1,
    glib_closure: FN,
) {
    let handle = RUNTIME.spawn(async move { tokio_fut.await });

    glib::MainContext::default().spawn_local_with_priority(priority, async move {
        glib_closure(handle.await.unwrap()).await
    });
}

pub fn run_stream<S, I, F>(mut stream: S, glib_closure: F)
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
