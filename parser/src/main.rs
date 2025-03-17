use parser::tokens::Element;

fn main() {
    let res = syn::parse_str::<Element>("<h1>Text</h1>");
    dbg!(res.unwrap());
}
