mod store;
mod serialize;

pub use store::Store;
pub use serialize::Serialize;
pub use store_derive::Store;
pub use serialize_derive::Serialize;
use std::fmt;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::HashMap;

#[derive(Debug)]
enum Operation {
    Write,
    Read(u32)
}

#[derive(Debug)]
pub struct Error {
    description: &'static str
}

impl Error {
    pub fn new(description: &'static str) -> Error {
        Error {
            description
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.description
    }
}

/// The `Database` struct contains everything used for a database.
#[derive(Default)]
pub struct Database {
    blocked: (Mutex<HashMap<String, Operation>>, Condvar)
}

impl Database {
    /// Creates a new database. The data is stored in the "data" directory at the project root, which is created automatically if it doesn't exist already.
    pub fn new() -> Database {
        Default::default()
    }

    /// Creates an entry in the database.
    pub fn create<I, T>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {                
        let key = object.key()?;
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();

        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if path.exists() {
            // return error if file exists
            Err(Box::new(Error::new("Entry already exists.")) as Box<dyn std::error::Error>)
        } else {
            // do the create
            let directory_string = format!("data/{}", T::name()?);
            let directory = Path::new(&directory_string);
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = File::create(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
            drop(file);
            Ok(())
        })();

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    /// Reads an entry from the database.
    pub fn read<I, T>(&self, object: &mut T) -> Result<(), Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {        
        let key = object.key()?;
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();

        // wait while key is blocked
        let mut readers = 0;
        while match (*guard).get(&key) {
            Some(operation) => match operation {
                Operation::Read(r) => {
                    readers = *r;
                    false
                },
                Operation::Write => true
            },
            None => false
        } {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Read(readers + 1));
        drop(guard);

        let output = (|| if !path.exists() {
            // return error if file doesn't exist
            Err(Box::new(Error::new("Entry doesn't exist.")) as Box<dyn std::error::Error>)
        } else {
            // do the read
            object.deserialize(&mut fs::read(path)?);
            Ok(())
        })();
        
        // acquire lock again and decrease readers
        let mut guard = lock.lock().unwrap();
        let updated_readers = match (*guard).get(&key) {
            Some(operation) => match operation {
                Operation::Read(r) => *r,
                Operation::Write => panic!("Operation is write but was reading")
            },
            None => panic!("Key not found but should be there")
        } - 1;
        if updated_readers == 0 {
            guard.remove(&key);
        } else {
            guard.insert(key, Operation::Read(updated_readers));
        }
        drop(guard);
        condvar.notify_all();

        output
    }

    /// Reads an entry from the database based off its ID.
    pub fn read_id<I, T>(&self, id: I) -> Result<T, Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {
        let mut object = T::with(id);
        self.read(&mut object)?;
        Ok(object)
    }

    /// Updates an entry in the database.
    pub fn update<I, T>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {
        let key = object.key()?;
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();
        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if !path.exists() {
            // return error if file doesn't exist
            Err(Box::new(Error::new("Entry doesn't exist.")) as Box<dyn std::error::Error>)
        } else {
            // do the update
            let directory_string = format!("data/{}", T::name()?);
            let directory = Path::new(&directory_string);
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = OpenOptions::new().write(true).open(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
            drop(file);
            Ok(())
        })();

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    /// Deletes an entry from the database.
    pub fn delete<I, T>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {
        let key = object.key()?;
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();
        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if !path.exists() {
            // return error if file doesn't exist
            Err(Box::new(Error::new("Entry doesn't exist.")) as Box<dyn std::error::Error>)
        } else {
            // do the delete
            fs::remove_file(path)?;
            Ok(())
        })();

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    
    /// Deletes an entry from the database based off its ID.
    pub fn delete_id<I, T>(&self, id: I) -> Result<(), Box<dyn std::error::Error>>
        where I: std::fmt::Display, T: Store<I> + Serialize
    {
        self.delete(&T::with(id))
    }

}