use super::Serialize;

pub trait Store<I>: Serialize + Default where I: std::fmt::Display {
    fn with(_: I) -> Self;
    fn name() -> Result<String, Box<dyn std::error::Error>>;
    fn id(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn key(&self) -> Result<String, Box<dyn std::error::Error>>;
}
