use rustsx::prelude::*;

// Test the new features

// 1. Text children by string variable
fn test_text_string() -> Dom {
    let text = "Hello, World!";
    rsx! {
        <p>{text}</p>
    }
}

// 2. Children by iterator with map
fn test_iterator_map() -> Dom {
    let chars = "hello";
    rsx! {
        <p>{"hello".chars().map(|c| rsx!(<strong>{c.to_string()}</strong>)).collect::<Vec<_>>()}</p>
    }
}

// 3. Multiple children with iterator
fn test_list_items() -> Dom {
    let items = vec!["one", "two", "three"];
    rsx! {
        <ul>
            {items.into_iter().map(|item| rsx!{
                <li>{item}</li>
            }).collect::<Vec<_>>()}
        </ul>
    }
}

// 4. Fully qualified imports - macro uses fully qualified paths internally
fn test_qualified_imports() -> Dom {
    rsx! {
        <button onclick={|_| {}}>
            {"Click me"}
        </button>
    }
}

// Compile test
fn main() {
    let _ = test_text_string();
    let _ = test_iterator_map();
    let _ = test_list_items();
    let _ = test_qualified_imports();
}
