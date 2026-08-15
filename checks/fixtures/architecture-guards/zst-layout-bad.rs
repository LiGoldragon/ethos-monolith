use std::marker::PhantomData as StdPhantomData;
use core::marker::PhantomData as CorePhantomData;

trait Behavior {
    fn act(&self);
}

struct Empty;
struct Braced {}
struct Tuple();
struct Pair(Empty, (), StdPhantomData<String>);
struct ZeroArray([u8; 0]);
struct ArrayOfEmpty([Empty; 4]);
struct Nested {
    pair: Pair,
    array: ZeroArray,
}
type PairAlias = Pair;
type PairAliasTransitive = PairAlias;

struct StdHolder(StdPhantomData<String>);
struct CoreHolder(CorePhantomData<String>);

mod reexports {
    pub use std::marker::PhantomData as ReexportedPhantomData;
}
use reexports::ReexportedPhantomData;
struct ReexportHolder(ReexportedPhantomData<String>);

mod direct_glob {
    use std::marker::*;
    struct DirectGlobHolder(PhantomData<String>);
    impl super::Behavior for DirectGlobHolder {
        fn act(&self) {}
    }
}

mod glob_scope {
    pub use core::marker::*;
    pub struct GlobHolder(PhantomData<String>);
    impl super::Behavior for GlobHolder {
        fn act(&self) {}
    }
}

struct PhantomData(u8);
struct LocalHolder(PhantomData);

struct Wrapper<T>(T);
struct Leaf<T>(T);
struct Outer<T>(Leaf<T>);
struct NestedOuter<T>(Outer<(T,)>);
struct ArrayOuter<T>([Leaf<T>; 1]);
struct ParenOuter<T>((Leaf<T>));
type UnitWrapper = Wrapper<()>;
type GenericAlias<T> = Wrapper<T>;
type GenericAliasTransitive<T> = GenericAlias<T>;
type LeafUnit = Leaf<()>;
type OuterAlias<T> = Outer<T>;
type OuterAliasTransitive<T> = OuterAlias<(T,)>;
type TupleAlias<T> = (Leaf<T>,);
type ArrayAlias<T> = [Leaf<T>; 1];
type ParenAlias<T> = (Leaf<T>);
type PhantomAlias<T> = StdPhantomData<T>;
type PhantomAliasTransitive<T> = PhantomAlias<T>;

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
impl Behavior for PairAlias {
    fn act(&self) {}
}
impl Behavior for PairAliasTransitive {
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
impl Behavior for StdHolder {
    fn act(&self) {}
}
impl Behavior for CoreHolder {
    fn act(&self) {}
}
impl Behavior for ReexportHolder {
    fn act(&self) {}
}
impl Behavior for UnitWrapper {
    fn act(&self) {}
}
impl Behavior for Wrapper<()> {
    fn act(&self) {}
}
impl Behavior for Leaf<()> {
    fn act(&self) {}
}
impl Behavior for Outer<()> {
    fn act(&self) {}
}
impl Behavior for NestedOuter<()> {
    fn act(&self) {}
}
impl Behavior for ArrayOuter<()> {
    fn act(&self) {}
}
impl Behavior for ParenOuter<()> {
    fn act(&self) {}
}
impl Behavior for LeafUnit {
    fn act(&self) {}
}
impl Behavior for OuterAlias<()> {
    fn act(&self) {}
}
impl Behavior for OuterAliasTransitive<()> {
    fn act(&self) {}
}
impl Behavior for TupleAlias<()> {
    fn act(&self) {}
}
impl Behavior for ArrayAlias<()> {
    fn act(&self) {}
}
impl Behavior for ParenAlias<()> {
    fn act(&self) {}
}
impl Behavior for GenericAlias<()> {
    fn act(&self) {}
}
impl Behavior for GenericAliasTransitive<()> {
    fn act(&self) {}
}
impl Behavior for PhantomAlias<()> {
    fn act(&self) {}
}
impl Behavior for PhantomAliasTransitive<()> {
    fn act(&self) {}
}
impl Behavior for Wrapper<u8> {
    fn act(&self) {}
}
impl Behavior for Leaf<u8> {
    fn act(&self) {}
}
impl Behavior for Outer<u8> {
    fn act(&self) {}
}
impl Behavior for NestedOuter<u8> {
    fn act(&self) {}
}
impl Behavior for ArrayOuter<u8> {
    fn act(&self) {}
}
impl Behavior for ParenOuter<u8> {
    fn act(&self) {}
}
impl Behavior for OuterAlias<u8> {
    fn act(&self) {}
}
impl Behavior for OuterAliasTransitive<u8> {
    fn act(&self) {}
}
impl Behavior for TupleAlias<u8> {
    fn act(&self) {}
}
impl Behavior for ArrayAlias<u8> {
    fn act(&self) {}
}
impl Behavior for ParenAlias<u8> {
    fn act(&self) {}
}
