use std::marker::PhantomData as StdPhantomData;

trait Behavior {
    fn act(&self);
}

struct Marker<const N: usize>;
struct LifetimeMarker<'a>;
struct PhantomMarker<'a, const N: usize>(StdPhantomData<&'a u8>);
struct ByteArray<const N: usize>([u8; N]);
struct Leaf<T>(T);
struct ZstArray<T, const N: usize>([Leaf<T>; N]);
struct Mixed<'a, T, const N: usize>(StdPhantomData<&'a T>);

type MarkerAlias<const N: usize> = Marker<{ N }>;
type LifetimeAlias<'a> = LifetimeMarker<'a>;
type PhantomAlias<'a, const N: usize> = PhantomMarker<'a, { N }>;
type MixedAlias<'a, T, const N: usize> = Mixed<'a, T, { N }>;
type ZstArrayAlias<T, const N: usize> = ZstArray<T, { N }>;
type ParenthesizedMarker<const N: usize> = Marker<{ (N) }>;
type ParenthesizedByteArray<const N: usize> = ByteArray<{ (N) }>;

impl Behavior for Marker<{ 0 }> {
    fn act(&self) {}
}
impl Behavior for Marker<{ 1 }> {
    fn act(&self) {}
}
impl Behavior for Marker<1> {
    fn act(&self) {}
}
impl Behavior for LifetimeMarker<'static> {
    fn act(&self) {}
}
impl Behavior for PhantomMarker<'static, { 42 }> {
    fn act(&self) {}
}
impl Behavior for ByteArray<{ 0 }> {
    fn act(&self) {}
}
impl Behavior for ZstArray<(), { 1 }> {
    fn act(&self) {}
}
impl Behavior for Mixed<'static, (), { 3 }> {
    fn act(&self) {}
}
impl Behavior for MarkerAlias<{ 2 }> {
    fn act(&self) {}
}
impl Behavior for LifetimeAlias<'static> {
    fn act(&self) {}
}
impl Behavior for PhantomAlias<'static, { 7 }> {
    fn act(&self) {}
}
impl Behavior for MixedAlias<'static, (), { 5 }> {
    fn act(&self) {}
}
impl Behavior for ZstArrayAlias<(), { 4 }> {
    fn act(&self) {}
}
impl Behavior for ParenthesizedMarker<{ 4 }> {
    fn act(&self) {}
}
impl Behavior for ParenthesizedByteArray<{ 0 }> {
    fn act(&self) {}
}
