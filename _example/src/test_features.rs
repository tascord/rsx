use rustsx::prelude::*;

// Test 1: Text children by variables
fn test_text_variable() -> Dom {
    let text = "Hello, World!";
    rsx! {
        <p>{text}</p>
    }
}

// Test 2: Children by iterator
fn test_iterator() -> Dom {
    rsx! {
        <p>{"hello".chars().map(|c| rsx!(<strong>{c.to_string()}</strong>)).collect::<Vec<_>>()}</p>
    }
}

// Test 3: Mixed content
fn test_mixed() -> Dom {
    let items = vec!["one", "two", "three"];
    rsx! {
        <div>
            <h1>{"Title"}</h1>
            <ul>
                {items.into_iter().map(|item| rsx! {
                    <li>{item}</li>
                }).collect::<Vec<_>>()}
            </ul>
        </div>
    }
}

// Test 4: Fully qualified imports
fn test_qualified() -> Dom {
    rsx! {
        <button onclick={|_| web_sys::console::log_1(&"clicked".into())}>
            {"Click me"}
        </button>
    }
}
