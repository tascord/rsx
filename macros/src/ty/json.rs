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

pub struct JsonMatchProps {
    pub path: String,
    pub js: String,
}

impl Parse for JsonMatchProps {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

        Ok(JsonMatchProps { path, js: rest })
    }
}
