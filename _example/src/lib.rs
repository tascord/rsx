use {
    futures_signals::signal::Mutable,
    rustsx::{
        dominator,
        prelude::{events::KeyUp, *},
    },
    web_sys::HtmlInputElement,
};

#[component]
fn greeting(text: Mutable<String>) -> Dom {
    rsx! {
        <div class="text-center mb-8">
            <h1 class="text-5xl font-bold bg-gradient-to-r from-purple-600 via-pink-600 to-blue-600 bg-clip-text text-transparent animate-gradient drop-shadow-lg">
                Hello {text}!
            </h1>
        </div>
    }
}

#[component]
fn button() -> dominator::Dom {
    let counter = Mutable::new(0usize);

    rsx! {
        <div class="flex flex-col items-center gap-4 p-6 bg-gradient-to-br from-blue-50 to-purple-50 rounded-xl shadow-lg">
            <button
                class="px-8 py-4 bg-gradient-to-r from-purple-500 to-pink-500 text-white font-bold rounded-lg shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200 active:scale-95"
                onclick={|_| counter.set(counter.get() + 1)}
            >
                Click Me!
            </button>
            <div class="text-2xl font-semibold text-gray-700">
                Clicked {counter} {counter.signal().map(|n| if n == 1 { "time" } else { "times" })}!
            </div>
        </div>
    }
}

fn demo() -> dominator::Dom {
    let name = Mutable::new("world".to_string());

    rsx! {
        <div class="min-h-screen bg-gradient-to-br from-indigo-100 via-purple-50 to-pink-100 p-8">
            <div class="max-w-4xl mx-auto space-y-8">
                <div class="text-center mb-12 animate-fade-in">
                    <h2 class="text-5xl font-bold text-gray-800 mb-4 tracking-tight">RSX Demo</h2>
                    <p class="text-xl text-gray-600">A rust tool for building web UIs</p>
                </div>

                <div class="bg-white rounded-2xl shadow-xl p-8 space-y-6 animate-slide-in hover:shadow-2xl transition-shadow duration-300">
                    <div class="border-b-2 border-purple-100 pb-4">
                        <h3 class="text-2xl font-semibold text-gray-800 mb-2">Dynamic Greeting</h3>
                        <p class="text-gray-600 text-sm">Type your name to see reactive updates in real-time</p>
                    </div>
                    <Greeting text={name.clone()}/>
                    <input
                        class="w-full px-6 py-4 text-lg border-2 border-purple-300 rounded-lg focus:outline-none focus:border-purple-500 focus:ring-4 focus:ring-purple-200 transition-all duration-200"
                        placeholder="Type your name here..."
                        onkeyup={|e: KeyUp| name.set(e.dyn_target::<HtmlInputElement>().unwrap().value())}
                    />
                </div>

                <div class="bg-white rounded-2xl shadow-xl p-8 space-y-6 animate-slide-in hover:shadow-2xl transition-shadow duration-300" style="animation-delay: 0.1s;">
                    <div class="border-b-2 border-blue-100 pb-4">
                        <h3 class="text-2xl font-semibold text-gray-800 mb-2">Interactive Counter</h3>
                        <p class="text-gray-600 text-sm">Click the button to increment the counter</p>
                    </div>
                    <Button/>
                </div>

                <div class="bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl shadow-xl p-8 text-white text-center animate-slide-in hover:shadow-2xl transition-all duration-300 hover:scale-105" style="animation-delay: 0.2s;">
                    <h3 class="text-2xl font-bold mb-2">Powered by RSX</h3>
                    <p class="text-purple-100 text-lg">"Fast", reactive, and written in Rust</p>
                </div>
            </div>
        </div>
    }
}

#[wasm_bindgen]
pub fn run_example() {
    console_error_panic_hook::set_once();

    // Use head BEFORE appending the main DOM element
    // This ensures scripts/styles are loaded in the correct order
    let head = web_sys::window().unwrap().document().unwrap().head().unwrap();

    // Add custom styles
    let style = web_sys::window().unwrap().document().unwrap().create_element("style").unwrap();

    style.set_inner_html(
        r#"
        @keyframes gradient {
            0%, 100% { background-position: 0% 50%; }
            50% { background-position: 100% 50%; }
        }
        .animate-gradient {
            background-size: 200% auto;
            animation: gradient 3s ease infinite;
        }
        @keyframes fade-in {
            from { opacity: 0; transform: translateY(-20px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .animate-fade-in {
            animation: fade-in 0.6s ease-out;
        }
        @keyframes slide-in {
            from { opacity: 0; transform: translateY(30px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .animate-slide-in {
            animation: slide-in 0.6s ease-out both;
        }
    "#,
    );

    head.append_child(&style).unwrap();

    let element = demo();
    let container = web_sys::window().unwrap().document().unwrap().get_element_by_id("app").unwrap();

    append_dom(&container, element);
}
