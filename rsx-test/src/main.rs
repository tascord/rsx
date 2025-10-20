use dominator::html;
use futures_signals::signal::Mutable;
use rsx_macro::jsx;

fn create_element() -> dominator::Dom {
    let counter = Mutable::new(0);
    jsx!(<div>{counter.signal()}</div>)
}

fn main() {}
