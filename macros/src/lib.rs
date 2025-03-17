use std::process::Command;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Arm, Expr, Variant, parse_macro_input, parse_str};
use ty::json::{JsonEnumProps, JsonMatchProps};

mod ty;

fn js_lines_from_file(js: String, path: String) -> Vec<String> {
    let output = &Command::new("node")
        .args([
            "-e",
            &format!("console.log((({js})(require({path}))).join('\\n'))",),
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() {
        panic!("Found stderr in js evaluation: {stderr}")
    }

    stdout.lines().map(|l| l.to_string()).collect()
}

#[proc_macro]
pub fn json_enum(ts: TokenStream) -> TokenStream {
    let props = parse_macro_input!(ts as JsonEnumProps);

    let variants = js_lines_from_file(props.js, props.path)
        .into_iter()
        .map(|v| {
            parse_str::<Variant>(&v)
                .expect(&format!("Failed to parse variant from JS\nVariant: {v}\n"))
        })
        .collect::<Vec<_>>();

    let name = props.ident;
    quote! {
        #[derive(Debug, Copy, Clone, strum::EnumString)]
        pub enum #name {
            #(#variants),*
        }
    }
    .into()
}

#[proc_macro]
pub fn json_match(ts: TokenStream) -> TokenStream {
    let props = parse_macro_input!(ts as JsonMatchProps);

    let items = js_lines_from_file(props.js, props.path)
        .into_iter()
        .map(|v| {
            parse_str::<Arm>(&v).expect(&format!("Failed to parse variant from JS\nArm: {v}\n"))
        })
        .collect::<Vec<_>>();

    quote! {
        match Self {
            #(#items),*
        }
    }
    .into()
}
