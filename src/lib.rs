mod database;
pub use crate::database::{Database, Store, SerializeBinary, Error};

#[cfg(test)]
mod tests {
    use super::{Database, Store, SerializeBinary};
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;

    #[derive(SerializeBinary, Store, Default, PartialEq, Debug)]
    struct Person {
        #[id] name: String,
        age: u16,
        text: String,
        vec: Vec::<u128>
    }
    
    impl Person {
        pub fn new(name: &'static str, age: u16) -> Person {
            Person {
                name: String::from(name),
                age,
                text: String::from("Hello\n\tWorld!"),
                vec: Vec::new()
            }
        }
    }

    #[derive(SerializeBinary, Store, Default)]
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

    /*
    #[test]
    fn lock_after_read() {
        let database = Database::new();
        Person::read(String::from("Something"), &database).expect_err("No error recieved");
        let peter = Person::new("Hans", 42);
        peter.create(&database).expect("Database create failed");
        Person::delete(String::from("Hans"), &database).expect("Database delete failed");
    }
    */

    #[test]
    fn crud_basics() {
        let database = Database::new();
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
    fn read_all() {
        let database = Database::new();
        database.create(&Person::new("Jakob", 56)).unwrap();
        database.create(&Person::new("Maria", 54)).unwrap();
        database.create(&Person::new("Josef", 51)).unwrap();
        assert_eq!(database.read_all::<Person>().unwrap().len(), 3);
        database.delete::<Person>(&String::from("Jakob")).unwrap();
        database.delete::<Person>(&String::from("Maria")).unwrap();
        database.delete::<Person>(&String::from("Josef")).unwrap();
    }

    /*
    #[test]
    fn crud_async() {
        executor::block_on(async {
            let database = Database::new();
            let mut peter_original = Person::new("Peter", 25);
            println!("{}", peter_original);
            database.create(&peter_original).await.expect("Database create failed");
            let mut peter_read = Person::new("Not Peter", 0);
            database.read(&mut peter_read).await.expect("Database read failed");
            assert_eq!(peter_read, peter_original);
            peter_original.age = 42;
            database.update(&peter_original).await.expect("Database update failed");
            database.read(&mut peter_read).await.expect("Database read failed");
            assert_eq!(peter_read, peter_original);
            database.delete(&peter_original).await.expect("Database delete failed");
        });
    }
    */
    
    
    #[test]
    fn crud_thread_times() {
        let database = Arc::new(Database::new());
        for i in 0..6 {
            let amount = (2 as u32).pow(i);
            let repetitions = 128 as u32 / amount;
            let start = Instant::now();
            let mut join_handles = Vec::new();
            for j in 0..amount {
                let db = Arc::clone(&database);
                join_handles.push(thread::spawn(move || {
                    for k in 0..repetitions {
                        let mut number = Number::new(j as u32 * repetitions + k as u32);
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
            println!("{} threads: {} ms", amount, start.elapsed().as_millis());
        }
    }
    
}