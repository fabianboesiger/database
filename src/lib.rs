mod database;
mod store;
mod bytes;
mod count;
mod error;

pub use crate::database::Database;
pub use store::Store;
pub use bytes::Bytes;
pub use count::Count;
pub use error::Error;
pub use store_derive::Store;
pub use bytes_derive::Bytes;

#[cfg(test)]
mod tests {
    use super::{Database, Store, Bytes};
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    #[derive(Bytes, Store, PartialEq, Debug, Clone)]
    #[from(Person2)]
    struct Person {
        #[id] name: String,
        age: u16,
    }

    impl From<Person2> for Person {
        fn from(person2: Person2) -> Person {
            Person {
                name: person2.name,
                age: person2.age
            }
        }
    }

    #[derive(Bytes, Store, PartialEq, Debug)]
    #[rename(Person)]
    struct Person2 {
        #[id] name: String,
        age: u16,
        text: String
    }
    
    impl From<Person> for Person2 {
        fn from(person: Person) -> Person2 {
            Person2 {
                name: person.name,
                age: person.age,
                text: String::new()
            }
        }
    }
    
    impl Person {
        pub fn new(name: &'static str, age: u16) -> Person {
            Person {
                name: String::from(name),
                age
            }
        }
    }

    #[derive(Bytes, Store)]
    struct Number {
        #[id] id: u32
    }

    impl Number {
        pub fn new(id: u32) -> Number {
            Number {
                id
            }
        }
    }

    #[derive(Bytes, Store)]
    struct AutoNumber {
        #[id] id: u32
    }

    impl AutoNumber {
        pub fn new() -> AutoNumber {
            AutoNumber {
                id: 0
            }
        }
    }
    /*
    #[test]
    fn encode_decode() {
        let id: u32 = 1234;
        let encoded = Database::encode(&id).unwrap();
        let decoded: u32 = Database::decode(&encoded).unwrap();
        println!("{} -> {} -> {}", id, encoded, decoded);
    }
    */
    #[test]
    fn serialize_deserialize() {
        let number: u32 = 1234;
        let serialized = number.serialize();
        let mut reversed = serialized.clone();
        reversed.reverse();
        let deserialized: u32 = Bytes::deserialize(&mut reversed).unwrap();
        println!("{} -> {:?} -> {}", number, serialized, deserialized);
    }

    #[test]
    fn basics() {
        let database = Database::new("data/basics");
        let mut peter_original = Person::new("Peter", 25);
        database.create(&peter_original).expect("Database create failed");
        let peter_read: Person = database.read(&String::from("Peter")).expect("Database read failed");
        assert_eq!(peter_read, peter_original);
        peter_original.age = 45;
        database.update(&peter_original).expect("Database update failed");
        let peter_read: Person = database.read(&String::from("Peter")).expect("Database read failed");
        assert_eq!(peter_read, peter_original);
        database.delete::<Person>(&peter_original.name).expect("Database delete failed");
    }

    #[test]
    fn from_old() {
        let database = Database::new("data/from-old");
        let peter: Person = Person::new("Peter", 25);
        let peter2: Person2 = peter.clone().into();
        database.create(&peter2).expect("Database create failed");
        let peter_read: Person = database.read(&String::from("Peter")).expect("Database read failed");
        assert_eq!(peter_read, peter);
        database.delete::<Person>(&String::from("Peter")).expect("Database delete failed");
    }

    #[test]
    fn read_all() {
        let database = Database::new("data/read-all");
        database.create(&Person::new("Jakob", 56)).unwrap();
        database.create(&Person::new("Maria", 54)).unwrap();
        database.create(&Person::new("Josef", 51)).unwrap();
        assert_eq!(database.read_all::<Person>().unwrap().len(), 3);
        database.delete_all::<Person>().unwrap();
    }

    #[test]
    fn auto_count() {
        let database = Database::new("data/auto-count");
        database.create_auto(&AutoNumber::new()).unwrap();
        database.create_auto(&AutoNumber::new()).unwrap();
        database.create_auto(&AutoNumber::new()).unwrap();
    }
    
    #[test]
    fn thread_times() {
        let database = Arc::new(Database::new("data/thread-times"));
        for i in 0..6 {
            let threads = (2 as u32).pow(i);
            let workload = 128 as u32 / threads;
            let start = Instant::now();
            let mut join_handles = Vec::new();
            for j in 0..threads {
                let db = Arc::clone(&database);
                join_handles.push(thread::spawn(move || {
                    for k in 0..workload {
                        let num = j as u32 * workload + k as u32;
                        let mut number = Number::new(num);
                        db.create(&number).expect("Database create failed");
                        number = db.read(&number.id).expect("Database read failed");
                        db.update(&number).expect("Database update failed");
                        db.delete::<Number>(&number.id).expect("Database delete failed");
                    }
                }));
            }
            for join_handle in join_handles {
                join_handle.join().unwrap();
            }
            println!("{} threads: {} ms", threads, start.elapsed().as_millis());
        }
    }
    
}