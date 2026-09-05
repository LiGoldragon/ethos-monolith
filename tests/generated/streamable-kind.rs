#![allow(dead_code)]
pub trait Streamable: super::Fillable {
    type Item: super::Serializable;
    const CAPACITY: protos::Integer;
    fn next(&mut self) -> std::option::Option<Self::Item>;
}
