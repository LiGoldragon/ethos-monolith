use std::marker::PhantomData;

trait Behavior {
    fn act(&self);
}

struct Empty;
struct Braced {}
struct Tuple();
struct Pair(Empty, (), PhantomData<String>);
struct ZeroArray([u8; 0]);
struct ArrayOfEmpty([Empty; 4]);
struct Nested {
    pair: Pair,
    array: ZeroArray,
}
type PairAlias = Pair;

impl Behavior for Empty {
    fn act(&self) {}
}
impl Behavior for Braced {
    fn act(&self) {}
}
impl Behavior for Tuple {
    fn act(&self) {}
}
impl Behavior for Pair {
    fn act(&self) {}
}
impl Behavior for ZeroArray {
    fn act(&self) {}
}
impl Behavior for ArrayOfEmpty {
    fn act(&self) {}
}
impl Behavior for Nested {
    fn act(&self) {}
}
impl Behavior for PairAlias {
    fn act(&self) {}
}
