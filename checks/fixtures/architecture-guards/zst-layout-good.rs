trait Behavior {
    fn act(&self);
}

struct Unit;
struct Data(u8);
struct Reference(&'static Unit);
struct Pointer(*const Unit);
struct ReferenceComposite {
    value: &'static Unit,
}
struct PointerComposite {
    value: *const Unit,
}
struct NonZeroComposite {
    value: Data,
    marker: Unit,
}

impl Behavior for Reference {
    fn act(&self) {}
}
impl Behavior for Pointer {
    fn act(&self) {}
}
impl Behavior for ReferenceComposite {
    fn act(&self) {}
}
impl Behavior for PointerComposite {
    fn act(&self) {}
}
impl Behavior for NonZeroComposite {
    fn act(&self) {}
}
impl Behavior for Data {
    fn act(&self) {}
}
