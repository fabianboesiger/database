pub trait Storable: Send {
    fn name() -> Result<String, Box<dyn std::error::Error>> where Self: Sized;
    fn id(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn key(&self) -> Result<String, Box<dyn std::error::Error>>;
}
