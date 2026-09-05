#![allow(dead_code)]
pub trait Processable<A: std::clone::Clone + std::marker::Send, B: serde::Serialize> {
    fn process(&self) -> protos::Text;
}
