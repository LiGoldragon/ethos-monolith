struct Holder(u8);
type Callback = extern "C" fn(u8);

#[cfg(test)]
fn test_only() {}

fn main() {}

trait Methods {
    fn method(&self);
    async fn asynchronous(&self);
    unsafe fn unsafe_method(&self);
}

impl Methods for Holder {
    fn method(&self) {}
    async fn asynchronous(&self) {}
    unsafe fn unsafe_method(&self) {}
}
