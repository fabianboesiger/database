pub mod database;
pub mod storable;

#[cfg(test)]
mod tests {
    use super::database::Database;
    use super::storable::Storable;
    use storable_derive::Storable;

    #[derive(Storable)]
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
    /*
    impl Storable for Person {
        fn name() -> &'static str {
            "person"
        }

        fn id<'a>(&self) -> &'a str {
            self.name
        }
    }
    */
    #[test]
    fn crud_single_threaded() {
        let peter = Person::new("Peter", 25);
        let database = Database::new();
        database.create(&peter).expect("Database create failed");
        database.update(&peter).expect("Database update failed");
    }
}