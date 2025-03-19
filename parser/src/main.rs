use dominator::DomBuilder;
use futures_signals::signal::Signal;
use parser::tokens::Element;
use web_sys::{wasm_bindgen::JsValue, HtmlElement};

fn main() {
    let res = syn::parse_str::<Element>("<h1 id=\"hello\">Text</h1>").unwrap();
    dbg!(res);
}