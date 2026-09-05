#![allow(dead_code)]
pub trait Summarizable {
    fn summarize(&self) -> protos::Text;
}
pub trait Fillable {
    fn push(&mut self, input: protos::Text) -> Result<protos::Integer, super::SinkError>;
    fn drain(&mut self) -> Vec<protos::Text>;
    fn create() -> Self;
}
