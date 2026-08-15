trait BadTrait {
    #[method_attribute_macro]
    fn method(&self);

    #[const_attribute_macro]
    const VALUE: usize;

    #[type_attribute_macro]
    type Value;
}

struct Bad;

impl BadTrait for Bad {
    #[evil::inline]
    fn method(&self) {}

    #[const_attribute_macro]
    const VALUE: usize = 0;

    #[type_attribute_macro]
    type Value = ();
}
