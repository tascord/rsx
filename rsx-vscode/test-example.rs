use rustsx::prelude::*;

#[component] 
fn example_component(name: String, count: usize) -> Dom {
    let class_name = "example-class";
    let is_disabled = count > 10;
    
    rsx! {
        <div class="container" id={format!("item-{}", count)}>
            <h1 class={class_name}>Hello {name}!</h1>
            <p>
                You have clicked {count} {if count == 1 { "time" } else { "times" }}
            </p>
            <button 
                class="btn btn-primary"
                disabled={is_disabled}
                onclick={|_| handle_click()}
            >
                Click me!
            </button>
            <div class="info">
                <span>Use {{double braces}} for literal text</span>
                <br />
                <span>Single braces for {format!("expressions: {}", 42)}</span>
            </div>
            <ul>
                <li>Item 1</li>
                <li>Item {2}</li>
                <li>Item {count + 1}</li>
            </ul>
        </div>
    }
}

fn handle_click() {
    println!("Button clicked!");
}

fn main() {
    let dom = example_component("World".to_string(), 5);
    // Mount to DOM...
}