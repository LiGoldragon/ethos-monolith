pub(crate) struct Empty {}
struct Tuple();

trait Acts {
    fn act(&self);
}

impl Acts for Empty {
    fn act(&self) {}
}

impl Acts for Tuple {
    fn act(&self) {}
}

impl Acts for (Empty) {
    fn act(&self) {}
}

struct Generic<T> {}

trait GenericActs {
    fn act(&self);
}

impl<T> GenericActs for Generic<T> {
    fn act(&self) {}
}

mod namespace {
    pub(crate) struct Namespaced;

    trait NamespacedActs {
        fn act(&self);
    }

    impl NamespacedActs for self::Namespaced {
        fn act(&self) {}
    }

    impl NamespacedActs for crate::namespace::Namespaced {
        fn act(&self) {}
    }
}
