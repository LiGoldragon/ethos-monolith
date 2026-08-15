macro_rules! make_items {
    () => {};
}

make_items!();
include!("macro-child.rs");

#[cfg_attr(test, derive(Clone))]
struct AttributeExpansion;
