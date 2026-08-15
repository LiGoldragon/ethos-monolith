trait Behavior {
    fn act(&self);
}

struct Bytes<const N: usize>([u8; N]);
struct GenericBytes<const N: usize>([u8; N]);

const EMPTY: usize = 0;
const ZERO_COMPUTED: usize = 1 - 1;
const CYCLE_A: usize = CYCLE_B;
const CYCLE_B: usize = CYCLE_A;

mod namespace {
    pub const ZERO: usize = 0;
}
use namespace::ZERO as IMPORTED_ZERO;

mod globbed {
    pub const GLOB_ZERO: usize = 0;
}
use globbed::*;

type BytesAlias<const N: usize> = Bytes<{ N }>;

impl Behavior for Bytes<0> {
    fn act(&self) {}
}
impl Behavior for Bytes<EMPTY> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ ZERO_COMPUTED }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ (1 - 1) }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ namespace::ZERO }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ IMPORTED_ZERO }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ GLOB_ZERO }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ CYCLE_A }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ 2 & 0 }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ 1 / 0 }> {
    fn act(&self) {}
}
impl<const N: usize> Behavior for GenericBytes<{ N }> {
    fn act(&self) {}
}
impl<const N: usize> Behavior for BytesAlias<{ N }> {
    fn act(&self) {}
}
