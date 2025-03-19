use proc_macro2::Literal;
use std::fmt::Debug;
use syn::{
    Expr, Ident, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

/// />
#[derive(Copy, Debug, Clone)]
pub struct ShortClose;
impl Parse for ShortClose {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;
        Ok(Self)
    }
}

/// </
#[derive(Copy, Debug, Clone)]
pub struct ShortOpen;
impl Parse for ShortOpen {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        Ok(Self)
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub props: Vec<Prop>,
    pub ident: Ident,
    pub children: Vec<Box<Node>>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
}

macro_rules! er {
    ($input: expr, $pat: literal, $( $rest: expr ),* ) => {
        syn::Error::new_spanned(
            $input.cursor().token_stream(),
            format!($pat, $($rest,)*),
        )
    };
}

macro_rules! try_rw {
    ($input: expr, $t: ty ) => {{
        let ff_input = $input.fork();
        match ff_input.parse::<$t>() {
            Err(_) => None,
            Ok(v) => {
                // println!("{} -> {}", $input.to_string(), ff_input.to_string());
                $input.advance_to(&ff_input);
                Some(v)
            }
        }
    }};
}

impl Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // {<} {element} {name=value}? {>} {children}? {</} {element} {>}
        // {<} {element} {name=value}? {/>}

        input
            .parse::<Token![<]>()
            .map_err(|e| er!(input, "Missing opening tri-brace: {:?}", e))?;

        let ident = input
            .parse::<Ident>()
            .map_err(|e| er!(input, "Missing element name: {:?}", e))?;

        let mut props = Vec::new();
        while let Some(prop) = try_rw!(input, Prop) {
            props.push(prop);
        }

        // Early close via {/>} skips parsing children
        let mut children = Vec::new();
        if input.parse::<ShortClose>().is_err() {
            input
                .parse::<Token![>]>()
                .map_err(|e| er!(input, "Missing closing tri-brace: {:?}", e))?;

            while let Some(el) = try_rw!(input, Node) {
                children.push(el);
            }
            input.parse::<ShortOpen>()?;
            let ident_2 = input
                .parse::<Ident>()
                .map_err(|e| er!(input, "Missing element name: {:?}", e))?;

            if ident_2.to_string() != ident.to_string() {
                dbg!("mismatching ident");
                panic!() // TODO
            }

            input.parse::<Token![>]>()?;
        }

        Ok(Element {
            props,
            ident,
            children: children.into_iter().map(|v| Box::new(v)).collect(),
        })
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(er!(input, "Nothing to parse... {}", ":("));
        }

        Ok(match try_rw!(input, Element) {
            Some(v) => Node::Element(v),
            None => {
                let text = input.step(|sc| {
                    let mut text = String::new();
                    let mut cursor = *sc;

                    loop {
                        if let Some((p, _)) = cursor.punct() {
                            if p.as_char() == '<' {
                                break;
                            }
                        }

                        if let Some((tt, c)) = cursor.token_tree() {
                            text.push_str(&tt.to_string());
                            cursor = c;
                        } else {
                            break;
                        }
                    }

                    Ok((text, cursor))
                })?;

                if text.is_empty() {
                    return Err(er!(input, "Empty text {}", ":("));
                }

                Node::Text(text)
            }
        })
    }
}

#[derive(Clone)]
pub struct Prop {
    pub name: Ident,
    pub value: Expr,
}

impl Debug for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Prop")
            .field("name", &self.name)
            .field("value", &"..")
            .finish()
    }
}

impl Parse for Prop {

    // {name}={value}
    // TODO: {name} (implies value=true)

    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input
            .parse::<Ident>()
            .map_err(|e| er!(input, "Missing prop name: {:?}", e))?;
        input
            .parse::<Token![=]>()
            .map_err(|e| er!(input, "Missing token equals '=': {:?}", e))?;

        let value = try_rw!(input, Expr)
            .or_else(|| {
                try_rw!(input, Literal).map(|v| {
                    Expr::Lit(syn::ExprLit {
                        attrs: Vec::new(),
                        lit: syn::Lit::Verbatim(v),
                    })
                })
            })
            .ok_or(er!(input, "Missing prop value: {:?}", ":("))?;

        Ok(Prop { name, value })
    }
}
