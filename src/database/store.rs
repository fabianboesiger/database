use super::Bytes;

pub trait Store: Bytes + Send + Sync {
    type Id: Bytes + Send + Sync;

    fn name() -> &'static str;
    fn id(&self) -> &Self::Id;
}