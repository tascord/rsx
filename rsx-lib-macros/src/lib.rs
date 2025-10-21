use {
    proc_macro::TokenStream,
    quote::{quote, ToTokens},
    std::{env, path::PathBuf, process::Command},
    syn::{Expr, Variant, parse_macro_input, parse_str},
    ty::json::{JsonEnumProps, JsonLines},
};

mod ty;

fn js_lines_from_file(js: String, path: String) -> Vec<String> {
    // Resolve the path relative to the macro crate's directory (lib-rsx-lib-macros)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(manifest_dir).join(&path);
    let absolute_path = full_path.canonicalize()
        .unwrap_or_else(|e| panic!("Failed to resolve path: {} (tried: {})\nError: {}", path, full_path.display(), e));
    let path_str = absolute_path.to_str().unwrap();
    
    let output = &Command::new("node")
        .args(["-e", &format!("console.log((({js})(require(\"{path_str}\"))).join('\\n'))",)])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() {
        panic!("Found stderr in js evaluation: {stderr}")
    }

    stdout.lines().map(|l| l.to_string()).collect()
}

fn eval_path_expr(path_expr: &Expr) -> String {
    // Try to evaluate common patterns like concat!() at compile time
    match path_expr {
        Expr::Lit(lit) => {
            if let syn::Lit::Str(s) = &lit.lit {
                return s.value();
            }
        }
        Expr::Macro(mac) => {
            // Handle concat!(file!(), "relative/path")
            if mac.mac.path.segments.last().map(|s| s.ident == "concat").unwrap_or(false) {
                let tokens = mac.mac.tokens.to_string();
                
                // Parse the concat arguments
                let parts: Vec<&str> = tokens.split(',').map(|s| s.trim()).collect();
                let mut result = String::new();
                
                for part in parts {
                    if part == "file!()" {
                        // We can't evaluate file!() at proc macro time, so we need to pass it through
                        // Instead, let's require the user to pass the literal path
                        panic!("Cannot evaluate file!() in proc macro context. Please use a string literal or environment variable pattern.");
                    } else if let Some(s) = part.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                        result.push_str(s);
                    }
                }
                
                return result;
            }
        }
        _ => {}
    }
    
    panic!("Could not evaluate path expression at compile time. Please use a string literal. Got: {:?}", path_expr.to_token_stream())
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
    let path_expr = props.path;
    
    // Try to evaluate the path expression
    let path = eval_path_expr(&path_expr);
    
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
