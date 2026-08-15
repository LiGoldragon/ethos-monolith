trait Behavior {
    fn act(&self);
}

struct Bytes<const N: usize>([u8; N]);

const ONE: usize = 1;
const TWO: usize = ONE + 1;
const THREE: usize = { (TWO) };
const ODD_MASK: usize = 3 & 1;
const SHIFTED: usize = 1 << 3;
const NEGATIVE: isize = -1;
const INVERTED: usize = !0;

mod namespace {
    pub const ONE: usize = 1;
    pub const TWO: usize = 1 | 1;
}
use namespace::ONE as IMPORTED_ONE;
use namespace::*;

impl Behavior for Bytes<1> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ ONE }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ TWO }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ THREE }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ ODD_MASK }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ SHIFTED }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ NEGATIVE }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ INVERTED }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ namespace::ONE }> {
    fn act(&self) {}
}
impl Behavior for Bytes<{ IMPORTED_ONE }> {
    fn act(&self) {}
}
