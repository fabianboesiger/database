pub mod database;
pub mod storable;

#[cfg(test)]
mod tests {
    use super::database::Database;
    use super::storable::Storable;
    use storable_derive::Storable;
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;
    use futures;

    #[derive(Storable, PartialEq, Clone, Debug)]
    struct Person {
        #[id] name: &'static str,
        age: u16
    }

    
    impl Person {
        pub fn new(name: &'static str, age: u16) -> Person {
            Person {
                name,
                age
            }
        }
    }

    #[derive(Storable)]
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
    
    #[test]
    fn crud_basics() {
        futures::executor::block_on(async {
            let database = Database::new();
            let mut peter_original = Person::new("Peter", 25);
            database.create(&peter_original).await.expect("Database create failed");
            let mut peter_read = Person::new("Not Peter", 0);
            database.read(&peter_read).await.expect("Database read failed");
            assert_eq!(peter_read, peter_original);
            peter_original.age = 42;
            database.update(&peter_original).await.expect("Database update failed");
            database.read(&peter_read).await.expect("Database read failed");
            assert_eq!(peter_read, peter_original);
            database.delete(&peter_original).await.expect("Database delete failed");
        });
    }
    
    #[test]
    fn crud_thread_times() {
        /*
        let database = Arc::new(Database::new());
        for i in 0..4 {
            let amount = (2 as u32).pow(i);
            let repetitions = 128 as u32 / amount;
            let start = Instant::now();
            let mut join_handles = Vec::new();
            for j in 0..amount {
                let db = Arc::clone(&database);
                join_handles.push(thread::spawn(move || {
                    for k in 0..repetitions {
                        let number = Number::new(j as u32 * repetitions + k as u32);
                        db.create(&number).expect("Database create failed");
                        db.read(&number).expect("Database read failed");
                        db.update(&number).expect("Database update failed");
                        db.delete(&number).expect("Database delete failed");
                    }
                }));
            }
            for join_handle in join_handles {
                join_handle.join().unwrap();
            }
            println!("{} threads: {} ms", amount, start.elapsed().as_millis());
        }
        */
    }
}