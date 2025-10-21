use {
    proc_macro::TokenStream,
    quote::quote,
    std::process::Command,
    syn::{Expr, Variant, parse_macro_input, parse_str},
    ty::json::{JsonEnumProps, JsonLines},
};

mod ty;

fn js_lines_from_file(js: String, path: String) -> Vec<String> {
    let output = &Command::new("node")
        .args(["-e", &format!("console.log((({js})(require(\"{path}\"))).join('\\n'))",)])
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
        .map(|v| parse_str::<Variant>(&v).unwrap_or_else(|_| panic!("Failed to parse variant from JS\nVariant: {v}\n")))
        .collect::<Vec<_>>();

    let name = props.ident;
    quote! {
        #[derive(Debug, Copy, Clone, strum::EnumString, strum::VariantNames)]
        pub enum #name {
            #(#variants),*
        }
    }
    .into()
}

#[proc_macro]
pub fn json(ts: TokenStream) -> TokenStream {
    let props = parse_macro_input!(ts as JsonLines);

    let path = {
        let path_str = props.path.clone();
        let path = std::path::Path::new(&path_str);
        if path.is_absolute() {
            path_str
        } else {
            let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(path.to_str().unwrap().trim_end_matches('"').trim_start_matches('"'));
            file.to_str().unwrap().to_string()
        }
    };

    let items = js_lines_from_file(props.js, path)
        .into_iter()
        .map(|v| parse_str::<Expr>(&v).unwrap_or_else(|_| panic!("Failed to parse variant from JS\nArm: {v}\n")))
        .collect::<Vec<_>>();

    quote! {
        [
            #(#items),*
        ]
    }
    .into()
}
