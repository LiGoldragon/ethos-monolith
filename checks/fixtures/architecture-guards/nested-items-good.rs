trait GoodTrait {
    #[doc = "an inert method attribute"]
    #[allow(dead_code)]
    fn method(&self);

    #[cfg(test)]
    const VALUE: usize;

    #[deprecated(note = "an inert associated type attribute")]
    type Value;
}

struct Good(u8);

impl GoodTrait for Good {
    #[inline]
    fn method(&self) {}

    #[allow(dead_code)]
    const VALUE: usize = 0;

    #[doc = "an inert associated type attribute"]
    type Value = ();
}
