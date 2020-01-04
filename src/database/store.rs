use super::SerializeBinary;

pub trait Store: SerializeBinary + Send + Sync {
    type ID: SerializeBinary + std::fmt::Display + Send + Sync;

    fn name() -> &'static str;
    fn id(&self) -> &Self::ID;
}
