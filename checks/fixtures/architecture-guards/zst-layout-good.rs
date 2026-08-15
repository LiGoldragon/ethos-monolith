use std::marker::PhantomData as StdPhantomData;

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
struct Wrapper<T>(T);
type ByteWrapper = Wrapper<u8>;
type Cycle = Cycle;
struct PhantomData(u8);
struct LocalHolder(PhantomData);
struct ImportedHolder(StdPhantomData<String>);

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
impl Behavior for Wrapper<u8> {
    fn act(&self) {}
}
impl Behavior for ByteWrapper {
    fn act(&self) {}
}
impl Behavior for LocalHolder {
    fn act(&self) {}
}
impl Behavior for Cycle {
    fn act(&self) {}
}
