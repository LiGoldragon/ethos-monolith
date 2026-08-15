use std::marker::PhantomData as StdPhantomData;

trait Behavior {
    fn act(&self);
}

struct Marker<const N: usize>;
struct LifetimeMarker<'a>;
struct ByteArray<const N: usize>([u8; N]);
struct Leaf<T>(T);
struct ZstArray<T, const N: usize>([Leaf<T>; N]);
struct Mixed<'a, T, const N: usize>(T, StdPhantomData<&'a T>);
struct Reference<'a>(&'a Marker<{ 0 }>);
struct Pointer(*const Marker<{ 0 }>);
struct GenericCycle<T>(GenericCycle<T>);

type MarkerAlias<const N: usize> = Marker<{ N }>;
type ByteArrayAlias<const N: usize> = ByteArray<{ N }>;
type MixedAlias<'a, T, const N: usize> = Mixed<'a, T, { N }>;
type GenericCycleAlias<T> = GenericCycle<T>;
type ParenthesizedByteArray<const N: usize> = ByteArray<{ (N) }>;

impl Behavior for ByteArray<{ 1 }> {
    fn act(&self) {}
}
impl Behavior for ByteArray<{ 2 }> {
    fn act(&self) {}
}
impl Behavior for ByteArrayAlias<{ 1 }> {
    fn act(&self) {}
}
impl Behavior for ParenthesizedByteArray<{ 1 }> {
    fn act(&self) {}
}
impl Behavior for ZstArray<u8, { 1 }> {
    fn act(&self) {}
}
impl Behavior for Mixed<'static, u8, { 0 }> {
    fn act(&self) {}
}
impl Behavior for MixedAlias<'static, u8, { 1 }> {
    fn act(&self) {}
}
impl Behavior for Reference<'static> {
    fn act(&self) {}
}
impl Behavior for Pointer {
    fn act(&self) {}
}
impl Behavior for GenericCycle<()> {
    fn act(&self) {}
}
impl Behavior for GenericCycleAlias<()> {
    fn act(&self) {}
}
