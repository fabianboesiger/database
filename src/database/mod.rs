mod store;
mod serialize_binary;
mod error;

pub use store::Store;
pub use serialize_binary::SerializeBinary;
pub use store_derive::Store;
pub use serialize_binary_derive::SerializeBinary;
pub use error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
enum Operation {
    Write,
    Read(u32)
}

/// The `Database` struct contains everything used for a database.
pub struct Database {
    path: PathBuf,
    blocked: (Mutex<HashMap<String, Operation>>, Condvar)
}

impl Database {
    /// Creates a new database. The data is stored in the "data" directory at the project root, which is created automatically if it doesn't exist already.
    pub fn new<P>(path: P) -> Database
        where P: AsRef<Path>
    {
        Database {
            path: path.as_ref().to_path_buf(),
            blocked: Default::default()
        }
    }

    pub fn id<I>(id: &I) -> Result<String, Error>
        where I: SerializeBinary + fmt::Display
    {
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut output = String::new();
        let mut sextets = Vec::<u8>::new();
        let bytes = id.serialize();
        
        for (i, &byte) in bytes.iter().enumerate() {
            match i % 3 {
                0 => {
                    sextets.push(byte & 0b00111111);
                    sextets.push((byte & 0b11000000) >> 6);
                },
                1 => {
                    let last = sextets.pop().unwrap();
                    sextets.push(last | ((byte & 0b00001111) << 2));
                    sextets.push((byte & 0b11110000) >> 4);
                },
                2 => {
                    let last = sextets.pop().unwrap();
                    sextets.push(last | ((byte & 0b00000011) << 4));
                    sextets.push((byte & 0b11111100) >> 2);
                },
                _ => unreachable!()
            }
        };
        if sextets.len() > 128 {
            return Err(Error::new(format!("ID \"{}\" exceeds maximum length", id)));
        }
        for &sextet in &sextets {
            output.push(alphabet.chars().skip(sextet as usize).next().expect("Alphabet out of range"));
        }

        Ok(output)
    }

    /// Creates an entry in the database.
    pub fn create<T>(&self, object: &T) -> Result<(), Error>
        where T: Store + SerializeBinary
    {           
        let key = format!("{}/{}", T::name(), Database::id(object.id())?);
        let path = self.path.clone().join(&key);
        let mut directory = path.clone();
        directory.pop();

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
            Err(Error::new(format!("Entry \"{}\" already exists", key)))
        } else {
            // do the create
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = File::create(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
            Ok(())
        })();

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    fn read_encoded<T>(&self, encoded: String) -> Result<T, Error>
        where T: Store + SerializeBinary
    {        
        let key = format!("{}/{}", T::name(), encoded);
        let path = self.path.clone().join(&key);

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
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
        } else {
            // do the read
            Ok(T::deserialize(&mut fs::read(path)?))
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

    /// Reads an entry from the database.
    pub fn read<T>(&self, id: &T::ID) -> Result<T, Error>
        where T: Store + SerializeBinary
    {        
        self.read_encoded(Database::id(id)?)
    }

    /// Reads all entries from the database
    pub fn read_all<T>(&self) -> Result<Vec<T>, Error>
        where T: Store + SerializeBinary
    {
        let mut result = Vec::new();
        match fs::read_dir(format!("data/{}", T::name())) {
            Ok(paths) => for path in paths {
                let encoded = String::from(path?.path().into_iter().last().unwrap().to_str().unwrap());
                result.push(self.read_encoded(encoded)?);
            },
            Err(_) => {}
        }

        Ok(result)
    }
    
    /// Updates an entry in the database.
    pub fn update<T>(&self, object: &T) -> Result<(), Error>
        where T: Store + SerializeBinary
    {
        let key = format!("{}/{}", T::name(), Database::id(object.id())?);
        let path = self.path.clone().join(&key);
        let mut directory = path.clone();
        directory.pop();

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
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
        } else {
            // do the update
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = OpenOptions::new().write(true).open(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
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
    pub fn delete<T>(&self, id: &T::ID) -> Result<(), Error>
        where T: Store + SerializeBinary
    {
        let key = format!("{}/{}", T::name(), Database::id(id)?);
        let path = self.path.clone().join(&key);

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
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
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

}