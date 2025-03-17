pub mod elements;
pub mod tokens;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::tokens::Element;

    #[test]
    fn it_works() {
        let res = syn::parse_str::<Element>("<h1>Text</h1>");
        dbg!(&res);

        assert!(res.is_ok())
    }
}
