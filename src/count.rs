/// This trait can be used on id's that should be automatically generated.
/// Both unsigned and signed integers implement this trait.
pub trait Count: std::cmp::PartialOrd + Default {
    fn next(&self) -> Self;
}

macro_rules! impl_SerializeBinary_for_primitives {
    ($($t:ty),+) => {
        $(
            impl Count for $t {
                fn next(&self) -> $t {
                    self + 1
                }
            }
        )*
    }
}

impl_SerializeBinary_for_primitives!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);