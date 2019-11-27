use std::fmt;

pub trait Storable: Sized + fmt::Display {
    fn name() -> String;
    fn id(&self) -> String;
    fn key(&self) -> String;
    fn from_bin(&self, bin: &[u8]) -> Result<(), ()>;
    fn to_bin(&self) -> Vec<u8>;
    fn from_string(&self, input: String);
}
