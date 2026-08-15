trait Behavior {
    fn act(&self);
}

mod source {
    pub struct Data {
        value: u8,
    }
    pub struct GlobData {
        value: u8,
    }
    pub mod nested {
        pub struct Unit {
            value: u8,
        }
    }
}

use source::Data as ImportedData;
pub use source::Data as ReexportedData;
type FirstAlias = ReexportedData;
type SecondAlias = FirstAlias;

impl Behavior for ImportedData {
    fn act(&self) {}
}
impl Behavior for SecondAlias {
    fn act(&self) {}
}

use source::*;
impl Behavior for GlobData {
    fn act(&self) {}
}
impl Behavior for nested::Unit {
    fn act(&self) {}
}

mod exports {
    pub use crate::source as alias;
}
use exports::*;
impl Behavior for alias::Data {
    fn act(&self) {}
}
use exports::alias as ImportedNamespace;
impl Behavior for ImportedNamespace::nested::Unit {
    fn act(&self) {}
}

mod shadow_source {
    pub struct Node;
}
use shadow_source::*;
struct Node {
    value: u8,
}
impl Behavior for Node {
    fn act(&self) {}
}

mod path_forms {
    pub struct CrateData {
        value: u8,
    }
    pub struct SuperData {
        value: u8,
    }
    pub mod source {
        pub struct MultiSuperData {
            value: u8,
        }
    }
    pub mod branch {
        pub struct SelfData {
            value: u8,
        }
        impl Behavior for self::SelfData {
            fn act(&self) {}
        }
        impl Behavior for super::SuperData {
            fn act(&self) {}
        }
        pub mod leaf {
            impl Behavior for super::super::source::MultiSuperData {
                fn act(&self) {}
            }
            impl Behavior for crate::path_forms::CrateData {
                fn act(&self) {}
            }
        }
    }
}

mod restricted_parent {
    mod restricted_source {
        pub(in super::super) struct GrandparentData {
            value: u8,
        }
    }
    pub mod nested {
        pub use super::restricted_source::*;
    }
}
use restricted_parent::nested::*;
impl Behavior for GrandparentData {
    fn act(&self) {}
}

#[path = "zst-resolution-good-child.rs"]
mod attributed;
