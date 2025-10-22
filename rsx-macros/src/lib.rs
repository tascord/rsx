use {
    heck::ToPascalCase,
    proc_macro::TokenStream,
    quote::quote,
    rsx_parser::tokens::Element,
    std::collections::HashSet,
    syn::{Expr, visit::Visit},
};

// Visitor to extract identifiers that might need cloning
struct IdentifierVisitor {
    identifiers: HashSet<String>,
    in_closure_params: bool,
    closure_params: HashSet<String>,
}

impl IdentifierVisitor {
    fn new() -> Self { Self { identifiers: HashSet::new(), in_closure_params: false, closure_params: HashSet::new() } }
}

impl<'ast> Visit<'ast> for IdentifierVisitor {
    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let old_in_params = self.in_closure_params;
        self.in_closure_params = true;

        for input in &node.inputs {
            syn::visit::visit_pat(self, input);
        }

        self.in_closure_params = old_in_params;

        syn::visit::visit_expr(self, &node.body);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        if self.in_closure_params {
            self.closure_params.insert(node.ident.to_string());
        }
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(ident) = node.path.get_ident() {
            let name = ident.to_string();
            if !self.closure_params.contains(&name) && !self.in_closure_params {
                self.identifiers.insert(name);
            }
        }
    }
}

fn extract_captured_variables(expr: &Expr) -> HashSet<String> {
    let mut visitor = IdentifierVisitor::new();
    visitor.visit_expr(expr);
    visitor.identifiers
}

fn modify_closure_to_use_clones(expr: &Expr, captured_vars: &HashSet<String>) -> proc_macro2::TokenStream {
    use syn::fold::{Fold, fold_expr};

    struct CloneReplacer<'a> {
        captured_vars: &'a HashSet<String>,
    }

    impl<'a> Fold for CloneReplacer<'a> {
        fn fold_expr_path(&mut self, mut node: syn::ExprPath) -> syn::ExprPath {
            // Only replace simple identifiers, not complex paths
            if let Some(ident) = node.path.get_ident() {
                let name = ident.to_string();
                if self.captured_vars.contains(&name) {
                    let new_ident = syn::Ident::new(&format!("{}_clone", name), ident.span());
                    node.path = syn::Path::from(new_ident);
                }
            }
            node
        }
    }

    let mut replacer = CloneReplacer { captured_vars };
    let modified_expr = fold_expr(&mut replacer, expr.clone());
    quote! { #modified_expr }
}

fn generate_component_code(element: &Element) -> proc_macro2::TokenStream {
    let component_name = &element.ident;
    let props_struct_name = syn::Ident::new(&format!("{}Props", component_name), component_name.span());

    // Generate props struct instantiation from attributes
    let mut prop_assignments = Vec::new();

    for prop in &element.props {
        let prop_name = &prop.name;
        let prop_value = &prop.value;

        prop_assignments.push(quote! {
            #prop_name: #prop_value
        });
    }

    // Call the component function with the props
    quote! {
        #component_name(#props_struct_name {
            #(#prop_assignments),*
        })
    }
}

fn generate_dom_code(element: &Element) -> proc_macro2::TokenStream {
    let tag_name = &element.ident;
    let tag_str = tag_name.to_string();

    // Check if this is a component (starts with uppercase)
    let first_char = tag_str.chars().next().unwrap_or('a');
    if first_char.is_uppercase() {
        // This is a component - generate component instantiation code
        return generate_component_code(element);
    }

    // Generate attributes for HTML elements
    let mut methods = Vec::new();

    for prop in &element.props {
        let attr_code = generate_attribute_code(prop, &tag_str);
        methods.push(attr_code);
    }

    // Handle style and script tags specially - their children should be treated as raw text
    // Exception: script tags with 'src' attribute should be treated as normal HTML elements
    let has_src_attr = tag_str == "script" && element.props.iter().any(|prop| prop.name == "src");
    
    if (tag_str == "style" || tag_str == "script") && !has_src_attr {
        if !element.children.is_empty() {
            let raw_content = extract_raw_content(&element.children);
            let text_method = quote! {
                .text(#raw_content)
            };
            methods.push(text_method);
        }
    } else {
        // Generate children if any (normal HTML elements)
        if !element.children.is_empty() {
            let mut children = Vec::new();
            for child in &element.children {
                let child_code = generate_child_code(child);
                children.push(child_code);
            }

            let children = children.into_iter().flatten().collect::<Vec<_>>();
            let children_method = quote! {
                .children(&mut [
                    #(#children),*
                ])
            };
            methods.push(children_method);
        }
    }

    // Generate dominator code structure
    quote! {
        html!(#tag_str, {
            #(#methods)*
        })
    }
}

fn generate_attribute_code(prop: &rsx_parser::tokens::Prop, tag_name: &str) -> proc_macro2::TokenStream {
    let attr_name = prop.name.to_string();
    let value = &prop.value;

    // Check if this is an event handler
    if attr_name.starts_with("on") {
        // Map HTML event names to dominator events
        let event_type = match attr_name.as_str() {
            "onclick" => quote! { dominator::events::Click },
            "onmousedown" => quote! { dominator::events::MouseDown },
            "onmouseup" => quote! { dominator::events::MouseUp },
            "onmousemove" => quote! { dominator::events::MouseMove },
            "ondblclick" => quote! { dominator::events::DoubleClick },
            "oncontextmenu" => quote! { dominator::events::ContextMenu },
            "onpointerover" => quote! { dominator::events::PointerOver },
            "onpointerenter" => quote! { dominator::events::PointerEnter },
            "onpointerdown" => quote! { dominator::events::PointerDown },
            "onpointermove" => quote! { dominator::events::PointerMove },
            "onpointerup" => quote! { dominator::events::PointerUp },
            "onpointercancel" => quote! { dominator::events::PointerCancel },
            "onpointerout" => quote! { dominator::events::PointerOut },
            "onpointerleave" => quote! { dominator::events::PointerLeave },
            "ongotpointercapture" => quote! { dominator::events::GotPointerCapture },
            "onlostpointercapture" => quote! { dominator::events::LostPointerCapture },
            "onkeydown" => quote! { dominator::events::KeyDown },
            "onkeyup" => quote! { dominator::events::KeyUp },
            "onfocus" => quote! { dominator::events::Focus },
            "onblur" => quote! { dominator::events::Blur },
            "onfocusin" => quote! { dominator::events::FocusIn },
            "onfocusout" => quote! { dominator::events::FocusOut },
            "ondragstart" => quote! { dominator::events::DragStart },
            "ondrag" => quote! { dominator::events::Drag },
            "ondragend" => quote! { dominator::events::DragEnd },
            "ondragover" => quote! { dominator::events::DragOver },
            "ondragenter" => quote! { dominator::events::DragEnter },
            "ondragleave" => quote! { dominator::events::DragLeave },
            "ondrop" => quote! { dominator::events::Drop },
            "oninput" => quote! { dominator::events::Input },
            "onbeforeinput" => quote! { dominator::events::BeforeInput },
            "onanimationstart" => quote! { dominator::events::AnimationStart },
            "onanimationiteration" => quote! { dominator::events::AnimationIteration },
            "onanimationcancel" => quote! { dominator::events::AnimationCancel },
            "onanimationend" => quote! { dominator::events::AnimationEnd },
            "onwheel" => quote! { dominator::events::Wheel },
            "onload" => quote! { dominator::events::Load },
            "onerror" => quote! { dominator::events::Error },
            "onscroll" => quote! { dominator::events::Scroll },
            "onscrollend" => quote! { dominator::events::ScrollEnd },
            "onsubmit" => quote! { dominator::events::Submit },
            "onresize" => quote! { dominator::events::Resize },
            "onselectionchange" => quote! { dominator::events::SelectionChange },
            "onchange" => quote! { dominator::events::Change },
            "ontouchcancel" => quote! { dominator::events::TouchCancel },
            "ontouchend" => quote! { dominator::events::TouchEnd },
            "ontouchmove" => quote! { dominator::events::TouchMove },
            "ontouchstart" => quote! { dominator::events::TouchStart },
            _ => quote! { web_sys::Event }, // fallback
        };

        // Generate event handler code with automatic cloning
        let captured_vars = extract_captured_variables(value);

        if captured_vars.is_empty() {
            // No captured variables, use original handler
            quote! {
                .event(move |event: #event_type| (#value)(event))
            }
        } else {
            // Generate clone statements for captured variables
            let clone_stmts: Vec<_> = captured_vars
                .iter()
                .map(|var| {
                    let var_ident = syn::Ident::new(var, proc_macro2::Span::call_site());
                    let clone_ident = syn::Ident::new(&format!("{}_clone", var), proc_macro2::Span::call_site());
                    quote! { let #clone_ident = #var_ident.clone(); }
                })
                .collect();

            // Create a new closure that uses the cloned variables
            let modified_closure = modify_closure_to_use_clones(value, &captured_vars);

            quote! {
                .event({
                    #(#clone_stmts)*
                    move |event: #event_type| (#modified_closure)(event)
                })
            }
        }
    } else {
        // Some attributes need to be set as properties instead of attributes
        // to trigger browser behavior correctly (e.g., script loading)
        let should_use_prop = matches!(
            (tag_name, attr_name.as_str()),
            ("script", "src") | ("img", "src") | ("iframe", "src") |
            ("video", "src") | ("audio", "src") | ("source", "src") |
            ("input", "value") | ("textarea", "value") | ("select", "value") |
            ("input", "checked") | ("option", "selected")
        );

        if should_use_prop {
            // Use .prop() for properties that need to trigger browser behavior
            quote! {
                .prop(#attr_name, #value)
            }
        } else {
            // Use .attr() for regular HTML attributes
            quote! {
                .attr(#attr_name, #value)
            }
        }
    }
}

fn extract_raw_content(children: &[Box<rsx_parser::tokens::Node>]) -> String {
    let mut content = String::new();
    
    for child in children {
        match child.as_ref() {
            rsx_parser::tokens::Node::Text(text) => {
                content.push_str(text);
            }
            rsx_parser::tokens::Node::Expression(expr) => {
                // For style/script tags, we want to preserve the expression syntax as-is
                content.push('{');
                content.push_str(&quote!(#expr).to_string());
                content.push('}');
            }
            rsx_parser::tokens::Node::Element(element) => {
                // Convert nested elements to text representation
                content.push('<');
                content.push_str(&element.ident.to_string());
                
                // Add attributes
                for prop in &element.props {
                    content.push(' ');
                    content.push_str(&prop.name.to_string());
                    content.push('=');
                    content.push('"');
                    let prop_value = &prop.value;
                    content.push_str(&quote!(#prop_value).to_string());
                    content.push('"');
                }
                
                if element.children.is_empty() {
                    content.push_str("/>");
                } else {
                    content.push('>');
                    content.push_str(&extract_raw_content(&element.children));
                    content.push_str("</");
                    content.push_str(&element.ident.to_string());
                    content.push('>');
                }
            }
        }
    }
    
    content
}

fn generate_child_code(child: &rsx_parser::tokens::Node) -> Vec<proc_macro2::TokenStream> {
    use rsx_parser::tokens::Node;
    match child {
        Node::Element(element) => vec![generate_dom_code(element)],
        Node::Text(text) => {
            let mut depth = 0;
            let mut buf = String::new();
            let mut doms = Vec::<proc_macro2::TokenStream>::new();
            let mut itr = text.chars().peekable();

            while let Some(c) = itr.next() {
                if c == '{' && itr.peek() != Some(&'{') {
                    depth += 1;
                    if depth == 1 {
                        // Starting a new expression - push any accumulated text
                        if !buf.is_empty() {
                            doms.push(quote! { Dom::text(#buf) });
                            buf.clear();
                        }
                    } else {
                        // Nested brace inside expression - add to buffer
                        buf.push(c);
                    }
                } else if c == '}' && itr.peek() != Some(&'}') {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            // Closing the expression - parse it
                            if !buf.is_empty() {
                                if let Ok(expr) = syn::parse_str::<Expr>(&buf) {
                                    doms.push(generate_child_code(&Node::Expression(expr)).into_iter().next().unwrap());
                                }
                                buf.clear();
                            }
                        } else {
                            // Nested brace inside expression - add to buffer
                            buf.push(c);
                        }
                    } else {
                        // Regular text
                        buf.push(c);
                    }
                } else {
                    buf.push(c);
                }
            }

            if depth != 0 {
                panic!("Expected '}}', found EOF")
            }

            if !buf.is_empty() {
                doms.push(quote! { Dom::text(#buf) });
                buf.clear();
            }

            doms
        }
        Node::Expression(expr) => {
            // Check if the expression already returns a signal type
            // Look for method calls that return signals: .signal(), .signal_cloned(), .signal_ref(), .map(), etc.
            let expr_str = quote!(#expr).to_string();

            if expr_str.contains(".signal()")
                || expr_str.contains(".signal_cloned()")
                || expr_str.contains(".signal_ref(")
                || expr_str.contains(".map(")
                || expr_str.contains(".map_ref(")
            {
                // Expression is already a signal, just ensure it produces String
                vec![quote! {
                    Dom::text_signal({
                        use futures_signals::signal::SignalExt;
                        (#expr).map(|x| format!("{}", x))
                    })
                }]
            } else {
                // Expression is a Mutable, call .signal() on it
                vec![quote! {
                    Dom::text_signal({
                        use futures_signals::signal::SignalExt;
                        #expr.signal_cloned().map(|x| format!("{}", x))
                    })
                }]
            }
        }
    }
}

#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    if let Ok(element) = syn::parse2::<Element>(input2) {
        let dom_code = generate_dom_code(&element);
        TokenStream::from(dom_code)
    } else {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Invalid JSX syntax - make sure to use proper HTML-like syntax: rsx!(<tag>content</tag>)",
        )
        .to_compile_error()
        .into()
    }
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    // Extract function name and convert to PascalCase for component and props struct names
    let fn_name = &input.sig.ident;
    let fn_name_str = fn_name.to_string();
    let component_name = syn::Ident::new(&fn_name_str.to_pascal_case(), fn_name.span());
    let props_struct_name = syn::Ident::new(&format!("{}Props", fn_name_str.to_pascal_case()), fn_name.span());

    // Extract parameters and generate props struct fields
    let mut prop_fields = Vec::new();
    let mut fn_params = Vec::new();
    let mut original_params = Vec::new();
    let mut generic_params = Vec::new();
    let mut where_clauses = Vec::new();

    for param in &input.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = param {
            // Extract parameter name
            let param_name = if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                &pat_ident.ident
            } else {
                continue;
            };

            // Check if this is a Mutable<T> parameter
            if is_mutable_type(&pat_type.ty) {
                let inner_type = extract_inner_type(&pat_type.ty);
                let generic_name =
                    syn::Ident::new(&format!("__{}", param_name.to_string().to_uppercase()), param_name.span());

                // Add generic parameter for the prop type
                generic_params.push(quote! { #generic_name });

                // Add where clause to constrain the generic
                where_clauses.push(quote! {
                    #generic_name: Into<::futures_signals::signal::Mutable<#inner_type>>
                });

                // Use the generic type in the props field
                prop_fields.push(quote! {
                    pub #param_name: #generic_name
                });

                // Convert the prop value to Mutable<T>
                fn_params.push(quote! { props.#param_name.into() });
            } else {
                // For non-Mutable types, pass through directly
                let field_type = &pat_type.ty;
                prop_fields.push(quote! {
                    pub #param_name: #field_type
                });

                fn_params.push(quote! { props.#param_name });
            }

            // Keep original parameters for the impl function
            original_params.push(pat_type);
        }
    }

    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_return = &input.sig.output;

    // Create a wrapper function that takes props and calls the impl
    let impl_fn_name = syn::Ident::new(&format!("{}_impl", fn_name), fn_name.span());

    // Generate the props struct with generics if needed
    let (props_generics, where_clause) = if !generic_params.is_empty() {
        (quote! { <#(#generic_params),*> }, quote! { where #(#where_clauses),* })
    } else {
        (quote! {}, quote! {})
    };

    let expanded = quote! {
        #[derive(Clone)]
        #[allow(non_snake_case)]
        #fn_vis struct #props_struct_name #props_generics
        #where_clause
        {
            #(#prop_fields),*
        }

        #[allow(non_snake_case)]
        #fn_vis fn #component_name #props_generics (props: #props_struct_name #props_generics) #fn_return
        #where_clause
        {
            #impl_fn_name(#(#fn_params),*)
        }

        #[allow(non_snake_case)]
        fn #impl_fn_name(#(#original_params),*) #fn_return {
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

// Helper function to check if a type is Mutable<T>
fn is_mutable_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Mutable";
    }
    false
}

// Helper function to extract T from Mutable<T> or return the type as-is
fn extract_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        // Check if this is Mutable<T>
        if segment.ident == "Mutable"
            && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return inner_ty.clone();
        }
    }
    ty.clone()
}
