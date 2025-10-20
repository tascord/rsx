use parser::tokens::Element;
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use syn::{ItemFn, parse_macro_input, parse2};

fn generate_dom_code(element: &Element) -> proc_macro2::TokenStream {
    let tag_name = &element.ident;
    let tag_str = tag_name.to_string();

    // Generate attributes
    let mut methods = Vec::new();

    for prop in &element.props {
        let attr_code = generate_attribute_code(prop);
        methods.push(attr_code);
    }

    // Generate children if any
    if !element.children.is_empty() {
        let mut children = Vec::new();
        for child in &element.children {
            let child_code = generate_child_code(child);
            children.push(child_code);
        }

        let children_method = quote! {
            .children(&mut [
                #(#children),*
            ])
        };
        methods.push(children_method);
    }

    // Generate dominator code structure
    quote! {
        html!(#tag_str, {
            #(#methods)*
        })
    }
}

fn generate_attribute_code(prop: &parser::tokens::Prop) -> proc_macro2::TokenStream {
    let attr_name = prop.name.to_string();
    let value = &prop.value;
    
    // Check if this is an event handler
    if attr_name.starts_with("on") {
        // Map HTML event names to dominator events
        let event_type = match attr_name.as_str() {
            "onclick" => quote! { dominator::events::Click },
            "onchange" => quote! { dominator::events::Change },
            "oninput" => quote! { dominator::events::Input },
            "onkeydown" => quote! { dominator::events::KeyDown },
            "onmouseenter" => quote! { dominator::events::MouseEnter },
            "onmouseleave" => quote! { dominator::events::MouseLeave },
            _ => quote! { web_sys::Event }, // fallback
        };
        
        // Generate event handler code
        quote! {
            .event(move |_: #event_type| (#value)())
        }
    } else {
        // Generate regular attribute code
        quote! {
            .attr(#attr_name, &#value)
        }
    }
}

fn generate_child_code(child: &parser::tokens::Node) -> proc_macro2::TokenStream {
    use parser::tokens::Node;
    match child {
        Node::Element(element) => {
            // Recursively generate code for child elements
            generate_dom_code(element)
        }
        Node::Text(text) => {
            // For text nodes, we'll create a text element
            // Note: dominator handles text differently - text is usually set via .text() on parent
            // For now, let's create a span with the text content
            quote! {
                html!("span", {
                    .text(#text)
                })
            }
        }
        Node::Expression(expr) => {
            // For expression nodes, generate signal-based text
            quote! {
                html!("span", {
                    .text_signal((#expr).map(|x| x.to_string()))
                })
            }
        }
    }
}

#[proc_macro]
pub fn jsx(input: TokenStream) -> TokenStream {
    // Convert TokenStream to proc_macro2::TokenStream for parsing
    let input2: proc_macro2::TokenStream = input.into();
    
    // Try to parse the JSX syntax using our parser
    if let Ok(element) = syn::parse2::<Element>(input2) {
        let dom_code = generate_dom_code(&element);
        TokenStream::from(dom_code)
    } else {
        // If parsing fails, return an error
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Invalid JSX syntax - make sure to use proper HTML-like syntax: jsx!(<tag>content</tag>)"
        ).to_compile_error().into()
    }
}

#[proc_macro_attribute]
pub fn rsx(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function that the attribute is applied to
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Transform the function body to handle JSX syntax
    let transformed_fn = transform_function_body(input_fn);
    
    let expanded = quote! {
        #transformed_fn
    };
    
    TokenStream::from(expanded)
}

fn transform_function_body(mut input_fn: ItemFn) -> ItemFn {
    // Get the function body
    let body = &mut input_fn.block.stmts;
    for stmt in body.iter_mut() {
        if let syn::Stmt::Local(local) = stmt {
            if let Some(init) = &mut local.init {
                // Transform the initializer expression
                let expr = &init.expr;
                let tokens: TokenStream = quote! { #expr }.into();
                let transformed = transform_tokens(tokens);
                if let Ok(new_expr) = syn::parse::<syn::Expr>(transformed) {
                    init.expr = Box::new(new_expr);
                }
            }
        }
    }
    input_fn
}

fn transform_tokens(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter().peekable();
    let mut result = Vec::new();
    
    while let Some(token) = tokens.next() {
        if let TokenTree::Punct(punct) = &token {
            if punct.as_char() == '<' {
                // Found potential JSX start, try to collect JSX tokens
                let mut jsx_tokens = vec![token];
                let mut depth = 1;
                
                while let Some(next_token) = tokens.next() {
                    jsx_tokens.push(next_token.clone());
                    
                    if let TokenTree::Punct(p) = &next_token {
                        match p.as_char() {
                            '<' => depth += 1,
                            '>' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                
                // Try to parse as JSX element
                let jsx_stream: proc_macro2::TokenStream = 
                    TokenStream::from_iter(jsx_tokens.clone()).into();
                
                if let Ok(element) = parse2::<Element>(jsx_stream) {
                    // Generate dominator code
                    let dom_code = generate_dom_code(&element);
                    let dom_tokens: TokenStream = dom_code.into();
                    result.extend(dom_tokens.into_iter());
                } else {
                    // Not valid JSX, keep original tokens
                    result.extend(jsx_tokens);
                }
            } else {
                result.push(token);
            }
        } else {
            result.push(token);
        }
    }
    
    TokenStream::from_iter(result)
}
