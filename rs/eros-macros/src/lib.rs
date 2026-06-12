// Enables feature flag documentation on things in docs.rs https://github.com/rust-lang/rust/issues/43781 http://doc.rust-lang.org/rustdoc/unstable-features.html#doccfg-and-docauto_cfg
#![cfg_attr(docsrs, feature(doc_cfg))]

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}