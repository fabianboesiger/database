use super::{Bytes, Count};

pub trait Store: Bytes + Send + Sync {
    type Id: Bytes + Send + Sync;

    fn name() -> &'static str;
    fn id(&self) -> &Self::Id;
}

pub trait Auto: Store where <Self as Auto>::Count: Into<<Self as Store>::Id> {
    type Count: Bytes + Send + Sync + Count;
}
