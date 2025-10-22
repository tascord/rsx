pub use {futures_signals, rsx_dominator as dominator, rsx_macros, wasm_bindgen};
use {rsx_dominator::Dom, web_sys::window};

pub mod prelude {
    pub use {
        rsx_dominator::*,
        rsx_macros::*,
        wasm_bindgen::{self, prelude::*},
    };
}

/// Appends a `dominator::Dom` to the `<head>`. <br>
/// **This function will panic if the window, document, or head can't be found.**
pub fn use_head(d: Dom) {
    let head = window()
        .expect("Failed to get window")
        .document()
        .expect("Failed to get document")
        .head()
        .expect("Failed to get <head/>");

    head.append_child(d.as_ref()).expect("Failed to append to head");
}
