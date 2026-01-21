pub use {futures_signals, rsx_dominator as dominator, rsx_macros, wasm_bindgen};
use {rsx_dominator::{Dom, DomBuilder}, web_sys::window};
use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::{SignalVec, SignalVecExt};

pub mod prelude {
    pub use {
        rsx_dominator::*,
        rsx_macros::*,
        wasm_bindgen::{self, prelude::*},
        crate::ApplyToDom, 
        crate::Fragment,
    };
}

// Appends a Dom to the head.
pub fn use_head(d: Dom) {
    let head = window()
        .expect("Failed to get window")
        .document()
        .expect("Failed to get document")
        .head()
        .expect("Failed to get <head/>");

    head.append_child(d.as_ref()).expect("Failed to append to head");
}

pub trait ApplyToDom<A> {
    fn apply_to_dom(self, builder: DomBuilder<A>) -> DomBuilder<A>;
}

// NOTE: impl for Dom is INHERENT.
// We DO NOT implement for &str/String to avoid conflicts.
// Users must use Dom::text("...") or Dom::text_signal(sig).

impl<A, T> ApplyToDom<A> for T
where
    A: AsRef<web_sys::Node>,
    T: SignalVec<Item = Dom> + 'static,
{
    fn apply_to_dom(self, builder: DomBuilder<A>) -> DomBuilder<A> {
        builder.children_signal_vec(self)
    }
}

pub struct Fragment(pub Vec<Dom>);

impl<A> ApplyToDom<A> for Fragment
where
    A: AsRef<web_sys::Node>,
{
    fn apply_to_dom(self, builder: DomBuilder<A>) -> DomBuilder<A> {
        builder.children(self.0)
    }
}
