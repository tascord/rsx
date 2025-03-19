use std::ops::Not;

use dominator::DomBuilder;
use futures_signals::signal::Signal;
use futures_signals::signal::SignalExt;
use itertools::Itertools;
use web_sys::{Element, HtmlElement, wasm_bindgen::JsValue};

pub fn apply<
    A: AsRef<HtmlElement> + AsRef<Element> + AsRef<JsValue>,
    B: Into<String> + Clone,
    C: Into<JsValue> + Clone,
>(
    el: DomBuilder<A>,
    key: B,
    value: C,
) {
    match should_set_as_prop(&el, key.clone(), value.clone()) {
        true => el.prop(key.into(), value),
        false => el.attr(
            key.into(),
            &Into::<JsValue>::into(value).as_string().unwrap_or_default(),
        ),
    };
}

pub fn bind<
    A: AsRef<HtmlElement> + AsRef<Element> + AsRef<JsValue>,
    B: Into<String> + Clone,
    C: Into<JsValue>,
    D: Signal<Item = C> + 'static,
>(
    el: DomBuilder<A>,
    key: B,
    value: D,
) {
    match should_set_as_prop(&el, key.clone(), JsValue::undefined()) {
        true => el.prop_signal(key.into(), value),
        false => el.attr_signal(
            key.into(),
            value.map(|v| Into::<JsValue>::into(v).as_string().unwrap_or_default()),
        ),
    };
}

mod attrs {
    use macros::json;

    /// Attr: &[Tags]
    pub const MAP: &'static [(&'static str, &'static [&'static str])] = json!(
        "./parser/mdn/attributes.json",
        r#"attrs => attrs.map(v => `("${v.attr}", &[${v.tags.map(v => `"${v}"`).join(", ")}])`)"#
    );
}

fn is_native_on(key: impl Into<String>) -> bool {
    if let Some((o, n, x)) = key.into().chars().take(3).collect_tuple() {
        o == 'o' && n == 'n' && x.is_lowercase()
    } else {
        false
    }
}

// https://github.com/vuejs/core/blob/958286e3f050dc707ad1af293e91bfb190bdb191/packages/runtime-dom/src/patchProp.ts#L69
fn should_set_as_prop<A: AsRef<HtmlElement>, B: Into<String>, C: Into<JsValue>>(
    el: &A,
    key: B,
    value: C,
) -> bool {
    // TODO: Handle is_svg
    let el: &HtmlElement = el.as_ref();
    let key: &str = &key.into();
    let value: JsValue = value.into();

    // these are enumerated attrs, however their corresponding DOM properties
    // are actually booleans - this leads to setting it with a string "false"
    // value leading it to be coerced to `true`, so we need to always treat
    // them as attributes.
    // Note that `contentEditable` doesn"t have this problem: its DOM
    // property is also enumerated string values.
    if key == "spellcheck" || key == "draggable" || key == "translate" {
        return false;
    }

    // #1787, #2840 form property on form elements is readonly and must be set as
    // attribute.
    if key == "form" {
        return false;
    }

    // #1526 <input list> must be set as attribute
    if key == "list" && el.tag_name() == "INPUT" {
        return false;
    }

    // #2766 <textarea type> must be set as attribute
    if key == "type" && el.tag_name() == "TEXTAREA" {
        return false;
    }

    // #8780 the width or height of embedded tags must be set as attribute
    if key == "width" || key == "height" {
        let tag = el.tag_name();
        if tag == "IMG" || tag == "VIDEO" || tag == "CANVAS" || tag == "SOURCE" {
            return false;
        }
    }

    // native onclick with string value, must be set as attribute
    if is_native_on(key) && value.is_string() {
        return false;
    }

    attrs::MAP
        .iter()
        .find(|v| v.0 == key)
        .is_some_and(|v| v.1.contains(&el.tag_name().to_lowercase().as_str()))
        .not()
}
