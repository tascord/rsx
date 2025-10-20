use parser::tokens::Element;

fn main() {
    let res = syn::parse_str::<Element>("<h1 id=\"hello\">Text</h1>").unwrap();
    dbg!(res);
}