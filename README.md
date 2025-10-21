<img width="200" height="200" align="left" style="float: left; margin: 0 10px 0 0;" alt="Icon" src="https://github.com/tascord/rsx/blob/main/icon.svg?raw=true"> 

# RustSX
## JSX-like syntax for Rust+WASM via [Dominator](https://github.com/Pauan/rust-dominator)

[![GitHub top language](https://img.shields.io/github/languages/top/tascord/rsx?color=0072CE&style=for-the-badge)](#)
[![Crates.io Version](https://img.shields.io/crates/v/rsx?style=for-the-badge)](https://crates.io/crates/rsx)
[![docs.rs](https://img.shields.io/docsrs/rsx?style=for-the-badge)](https://docs.rs/rsx)

## Why??????????
A question i ask myself every day man

## What
RSX!() is a tool that allows you to write JSX-like syntax in Rust for building web UIs with Dominator. It aims to make the process of creating web components more intuitive, epsecially for those familiar with JSX in frameworks like React. It also aims to remove a lot of the pain points for large-scale projects, such as managing signals and props.

## Usability
This project is still VERY early in its life. Still lacking support for many things, and likely to have breaking changes. Use at your own risk, or contrib to work on solidifying things :).

## Example Usage
Have a look at the [example project](./_example/) & [runner script](./example.sh) for more details.
```rust

fn basic() -> dominator::Dom {
    let count = Mutable::new(0);

    rsx! {
        <button onclick={|_event| count.set(count.get().add(1))}> // Clones as needed
            Clicked {count} times. // Automatically coerces Mutable to signal
        </button>
    }
}

#[component]
fn component(title: Mutable<String>) -> dominator::Dom {
    rsx! {
        <div class="container">
            <h1>{title}</h1>
        </div>
    }
}

// Behind the scenes makes a struct for props, and renames the function to be coherent with JSX-like casing

fn demo_component() -> dominator::Dom {
    let title = Mutable::new("Hello, World!".to_string());

    rsx! {
        <div>
            <Component title={title.clone()} /> // Pass props, matching type
            <Component title="String!" /> // Pass props, auto-coerced
        </div>
    }
}

```

## Contributing
Contributions are welcome if you're up for it.
### Map
- [`./dominator`](./dominator) - Dominator vendor
- [`./rsx-lib-macros`](./rsx-lib-macros) - Macros used within the library to make things easier (e.g. parsing MDN docs)
- [`./parser`](./parser) - Lib tools (should probably be renamed as its not parsing stuff anymore)
- [`./rsx-macro`](./rsx-macro) - The macro itself
- [`./rsx`](./rsx) - The wrapper library that exports everything we need 