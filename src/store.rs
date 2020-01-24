use super::Bytes;

/// This trait has to be implemented on a struct that should be stored.
/// Note that the `Bytes` trait has to be implemented too as `Store` is a supertrait of `Bytes`.
/// Implement both using `#[derive(Store, Bytes)]`.
/// Further, all fields of a struct that implements `Store` have to implement `Bytes`.
pub trait Store: Bytes + Send + Sync {
    type Id: Bytes + Send + Sync;
    const NAME: &'static str;
    fn id(&self) -> &Self::Id;
}