use super::Serialize;

pub trait Store: Serialize + Default + Send + Sync {
    type ID: Serialize + std::fmt::Display + Send + Sync;

    fn name() -> &'static str;
    fn id(&self) -> Self::ID;
}
