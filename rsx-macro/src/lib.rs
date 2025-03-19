use parser::tokens::Element;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn rsx(ts: TokenStream) -> TokenStream {
    ts
}

fn parse_elem(e: Element) -> TokenStream {


    quote! {
        
    }
}