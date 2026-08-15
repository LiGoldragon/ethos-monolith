pub(crate) fn crate_visible() {}
pub async fn asynchronous() {}
const fn constant() {}
unsafe fn unsafe_function() {}

#[cfg(any(test, feature = "fixture"))]
fn partially_test_only() {}

mod nested {
    #[inline]
    pub(crate) async unsafe fn inline_nested() {}
}

extern "C" fn external_function() {}
pub fn r#raw_function() {}
