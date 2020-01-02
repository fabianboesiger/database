use super::Serialize;

pub trait Store: Serialize + Default {
    type ID: Serialize + std::fmt::Display;

    fn name() -> &'static str;
    fn id(&self) -> Self::ID;
}
