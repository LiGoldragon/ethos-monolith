trait Behavior {
    fn act(&self);
}

mod source {
    pub struct AliasZst;
    pub struct GlobZst;
    pub mod nested {
        pub struct Unit;
    }
}

use source::AliasZst as ImportedZst;
pub use source::AliasZst as ReexportedZst;
type FirstAlias = ReexportedZst;
type SecondAlias = FirstAlias;

impl Behavior for ImportedZst {
    fn act(&self) {}
}
impl Behavior for SecondAlias {
    fn act(&self) {}
}

use source::*;
impl Behavior for GlobZst {
    fn act(&self) {}
}
impl Behavior for nested::Unit {
    fn act(&self) {}
}

mod exports {
    pub use crate::source as alias;
}
use exports::*;
impl Behavior for alias::AliasZst {
    fn act(&self) {}
}
use exports::alias as ImportedNamespace;
impl Behavior for ImportedNamespace::AliasZst {
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
    pub struct CrateZst;
    pub struct SuperZst;
    pub mod source {
        pub struct MultiSuperZst;
    }
    pub mod branch {
        pub struct SelfZst;
        impl Behavior for self::SelfZst {
            fn act(&self) {}
        }
        impl Behavior for super::SuperZst {
            fn act(&self) {}
        }
        pub mod leaf {
            impl Behavior for super::super::source::MultiSuperZst {
                fn act(&self) {}
            }
            impl Behavior for crate::path_forms::CrateZst {
                fn act(&self) {}
            }
        }
    }
}

mod restricted_parent {
    mod restricted_source {
        pub(in super::super) struct GrandparentZst;
    }
    pub mod nested {
        pub use super::restricted_source::*;
    }
}
use restricted_parent::nested::*;
impl Behavior for GrandparentZst {
    fn act(&self) {}
}

#[path = "zst-resolution-bad-child.rs"]
mod attributed;
