pub trait Bytes {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(_: &mut Vec<u8>) -> Self;
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

                fn deserialize(bytes: &mut Vec<u8>) -> $t {
                    const SIZE: usize = std::mem::size_of::<$t>();
                    let mut my_bytes = [0; SIZE];
                    for i in 0..SIZE {
                        let byte = bytes.pop().unwrap();
                        my_bytes[i] = byte;
                    }
                    <$t>::from_le_bytes(my_bytes)
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

    fn deserialize(bytes: &mut Vec<u8>) -> bool {
        bytes.pop().unwrap() == 1
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

    fn deserialize(bytes: &mut Vec<u8>) -> Vec<S> {
        let mut output = Vec::new();
        let size = u64::deserialize(bytes);
        for _ in 0..size {
            output.push(S::deserialize(bytes));
        }
        output
    }
}

impl Bytes for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(format!("{}\0", self).as_bytes());
        bytes
    }

    fn deserialize(bytes: &mut Vec<u8>) -> String {
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
        String::from(std::str::from_utf8(&my_bytes).unwrap())
    }
}
