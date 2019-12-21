pub trait Store {
    fn name() -> Result<String, Box<dyn std::error::Error>>;
    fn id(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn key(&self) -> Result<String, Box<dyn std::error::Error>>;
}
