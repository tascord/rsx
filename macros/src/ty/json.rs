use proc_macro2::Literal;
use syn::{Ident, Token, buffer::Cursor, parse::Parse};

pub struct JsonEnumProps {
    pub ident: Ident,
    pub path: String,
    pub js: String,
}

impl Parse for JsonEnumProps {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse().expect("Expected ident for enum name");
        input.peek(Token![,]).then_some(()).inspect(|_| {
            let _ = input.parse::<Token![,]>();
        });

        let path = input
            .parse::<Literal>()
            .expect("Expected literal for path")
            .to_string();

        input.peek(Token![,]).then_some(()).inspect(|_| {
            let _ = input.parse::<Token![,]>();
        });

        let mut rest = String::new();
        input
            .step(|v| {
                rest = v.token_stream().to_string();
                Ok(((), Cursor::empty()))
            })
            .expect("Failed to get remaining tokens");

        Ok(JsonEnumProps {
            ident,
            path,
            js: rest,
        })
    }
}

pub struct JsonLines {
    pub path: String,
    pub js: String,
}

impl Parse for JsonLines {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input
            .parse::<Literal>()
            .expect("Expected literal for path")
            .to_string();

        input.peek(Token![,]).then_some(()).inspect(|_| {
            let _ = input.parse::<Token![,]>();
        });

        let mut rest = input.to_string();
        input.step(|_| Ok(((), Cursor::empty()))).unwrap();

        if let Some(left) = rest.strip_prefix("r#\"") {
            rest = left.strip_suffix("\"#").unwrap().to_string();
        }

        Ok(JsonLines { path, js: rest })
    }
}
