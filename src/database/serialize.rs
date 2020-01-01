pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(&mut self, _: &mut Vec<u8>);
}

macro_rules! impl_Serializable_for_primitives {
    ($($t:ty),+) => {
        $(
            impl Serialize for $t {
                fn serialize(&self) -> Vec<u8> {
                    let mut bytes = Vec::new();
                    bytes.extend_from_slice(&self.to_le_bytes()[..]);
                    bytes
                }

                fn deserialize(&mut self, bytes: &mut Vec<u8>) {
                    const SIZE: usize = std::mem::size_of::<$t>();
                    let mut my_bytes = [0; SIZE];
                    for i in 0..SIZE {
                        let byte = bytes.pop().unwrap();
                        my_bytes[i] = byte;
                    }
                    *self = <$t>::from_le_bytes(my_bytes);
                }
            }
        )*
    }
}

impl_Serializable_for_primitives!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

impl Serialize for bool {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(if *self { 1 } else { 0 });
        bytes
    }

    fn deserialize(&mut self, bytes: &mut Vec<u8>) {
        *self = bytes.pop().unwrap() == 1;
    }
}

impl<S: Serialize + Default> Serialize for Vec<S> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.append(&mut (self.len() as u64).serialize());
        for element in self {
            bytes.append(&mut element.serialize());
        }
        bytes
    }

    fn deserialize(&mut self, bytes: &mut Vec<u8>) {
        let mut size: u64 = 0;
        size.deserialize(bytes);
        for _ in 0..size {
            let mut element = S::default();
            element.deserialize(bytes);
            self.push(element);
        }
    }
}

impl Serialize for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(format!("{}\0", self).as_bytes());
        bytes
    }

    fn deserialize(&mut self, bytes: &mut Vec<u8>) {
        let mut my_bytes = Vec::new();
        loop {
            let byte = bytes.pop().unwrap();
            if byte == b'\0' {
                break;
            }
            my_bytes.push(byte);
        }
        *self = String::from(std::str::from_utf8(&my_bytes).unwrap());
    }
}
