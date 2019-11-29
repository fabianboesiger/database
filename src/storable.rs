use std::fmt;

pub trait Storable: Send + fmt::Display {
    fn name() -> String where Self: Sized;
    fn id(&self) -> String;
    fn key(&self) -> String;
    fn from_bin(&mut self, _: Vec<u8>);
    fn to_bin(&self) -> Vec<u8>;
}
