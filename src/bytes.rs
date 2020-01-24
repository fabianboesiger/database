use super::Error;

/// The `Bytes` trait has to be implemented in order to use the `Store` trait.
pub trait Bytes {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(_: &mut Vec<u8>) -> Result<Self, Error> where Self: Sized;
    // TODO: Move signature computation to compile time.
    fn signature() -> String;
    fn hash() -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hasher::write(&mut hasher, Self::signature().as_bytes());
        std::hash::Hasher::finish(&hasher)
    }
}

macro_rules! impl_SerializeBinary_for_primitives {
    ($($t:ty),+) => {
        $(
            impl Bytes for $t {
                fn serialize(&self) -> Vec<u8> {
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&self.to_le_bytes()[..]);
                    bytes
                }

                fn deserialize(bytes: &mut Vec<u8>) -> Result<$t, Error> {
                    const SIZE: usize = std::mem::size_of::<$t>();
                    let mut my_bytes = [0; SIZE];
                    for i in 0..SIZE {
                        let byte = bytes.pop().unwrap();
                        my_bytes[i] = byte;
                    }
                    Ok(<$t>::from_le_bytes(my_bytes))
                }

                fn signature() -> String {
                    String::from(stringify!($t))
                }
            }
        )*
    }
}

impl_SerializeBinary_for_primitives!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl Bytes for bool {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(if *self { 1 } else { 0 });
        bytes
    }

    fn deserialize(bytes: &mut Vec<u8>) -> Result<bool, Error> {
        Ok(bytes.pop().unwrap() == 1)
    }

    fn signature() -> String {
        String::from("bool")
    }
}

impl<S: Bytes> Bytes for Vec<S> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.append(&mut (self.len() as u64).serialize());
        for element in self {
            bytes.append(&mut element.serialize());
        }
        bytes
    }

    fn deserialize(bytes: &mut Vec<u8>) -> Result<Vec<S>, Error> {
        let mut output = Vec::new();
        let size = u64::deserialize(bytes)?;
        for _ in 0..size {
            output.push(S::deserialize(bytes)?);
        }
        Ok(output)
    }

    fn signature() -> String {
        format!("Vec<{}>", S::signature())
    }
}

impl Bytes for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(format!("{}\0", self).as_bytes());
        bytes
    }

    fn deserialize(bytes: &mut Vec<u8>) -> Result<String, Error> {
        let mut my_bytes = Vec::new();
        loop {
            match bytes.pop() {
                Some(byte) => {
                    if byte == b'\0' {
                        break;
                    }
                    my_bytes.push(byte);
                },
                None => { break; }
            }
        }
        Ok(String::from(std::str::from_utf8(&my_bytes).unwrap()))
    }

    fn signature() -> String {
        String::from("String")
    }
}
