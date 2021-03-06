use super::{Error, Bytes, Store, Count};
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::HashMap;

//const BASE: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const BASE: &'static str = "0123456789abcdef";

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
    /*
    fn encode<I>(id: &I) -> Result<String, Error>
        where I: Bytes
    {
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
            return Err(Error::new(format!("Id exceeds maximum length")));
        }
        for &sextet in &sextets {
            output.push(BASE.chars().skip(sextet as usize).next().expect("Alphabet out of range"));
        }

        Ok(output)
    }
    
    fn decode<I>(string: &String) -> Result<I, Error>
        where I: Bytes
    {
        let mut bytes = Vec::<u8>::new();
        
        for (i, c) in string.chars().enumerate() {
            let byte = BASE.find(c).ok_or_else(|| Error::new(format!("Invalid character")))? as u8;
            match i % 4 {
                0 => {
                    bytes.push(byte);
                },
                1 => {
                    let last = bytes.pop().unwrap();
                    bytes.push(last | (byte << 6));
                    bytes.push(byte >> 2);
                },
                2 => {
                    let last = bytes.pop().unwrap();
                    bytes.push(last | (byte << 4));
                    bytes.push(byte >> 4);
                },
                3 => {
                    let last = bytes.pop().unwrap();
                    bytes.push(last | (byte << 2));
                },
                _ => unreachable!()
            }
        };

        bytes.reverse();
        Ok(I::deserialize(&mut bytes)?)
    }
    */

    fn encode<I>(id: &I) -> Result<String, Error>
        where I: Bytes
    {
        let mut output = String::new();
        let bytes = id.serialize();
        
        for byte in &bytes {
            output.push(BASE.chars().nth((byte / BASE.len() as u8) as usize).unwrap());
            output.push(BASE.chars().nth((byte % BASE.len() as u8) as usize).unwrap());

            if output.len() > 128 {
                return Err(Error::new(format!("Id is too long.")));
            }
        }

        if output.len() == 0 {
            return Err(Error::new(format!("Id is too short.")));
        }

        Ok(output)
    }
    
    fn decode<I>(string: &String) -> Result<I, Error>
        where I: Bytes
    {
        let mut bytes = Vec::<u8>::new();
        let mut chars = string.chars();

        loop {
            let c1 = if let Some(c) = chars.next() {
                c
            } else {
                break
            };
            let c2 = chars.next().ok_or_else(|| Error::new(format!("Invalid entry name.")))?;

            let b1 = BASE.find(c1).ok_or_else(|| Error::new(format!("Invalid character in entry name.")))? as u8;
            let b2 = BASE.find(c2).ok_or_else(|| Error::new(format!("Invalid character in entry name.")))? as u8;
        
            bytes.push(b1 * 16 + b2);
        }

        bytes.reverse();

        Ok(I::deserialize(&mut bytes)?)
    }

    pub fn exists<T>(&self, object: &T) -> Result<bool, Error>
        where T: Store
    {
        let key = format!("{}/{}", T::NAME, Database::encode(object.id())?);
        let path = self.path.clone().join(&key);
        Ok(path.exists())
    }

    fn next_id<T>(&self) -> T::Id
        where T: Store, T::Id: Count
    {
        let mut output = Default::default();
        if let Ok(paths) = std::fs::read_dir(self.path.clone().join(T::NAME)) {
            for path in paths {
                let encoded = String::from(path.unwrap().path().into_iter().last().unwrap().to_str().unwrap());
                let value = Database::decode::<T::Id>(&encoded).unwrap();
                if value >= output {
                    output = value.next();
                }
            }
        }
        output
    }

    fn create_id<T>(&self, object: &T, id: &T::Id) -> Result<(), Error>
        where T: Store
    {
        let key = format!("{}/{}", T::NAME, Database::encode(id)?);
        let path = self.path.clone().join(&key);
        let mut directory = path.clone();
        directory.pop();

        // Acquire lock.
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();

        // Wait while key is blocked.
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // If key isn't locked, insert it into the locked set and release lock.
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if path.exists() {
            // Return error if file exists.
            Err(Error::new(format!("Entry \"{}\" already exists", key)))
        } else {
            // Do the create.
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = File::create(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
            Ok(())
        })();

        // Acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    pub fn create_auto<T>(&self, object: &T) -> Result<(), Error>
        where T: Store, T::Id: Count
    {
        self.create_id(object, &self.next_id::<T>().into())
    }
    
    /// Creates an entry in the database.
    pub fn create<T>(&self, object: &T) -> Result<(), Error>
        where T: Store
    {
        self.create_id(object, &object.id())
    }

    fn read_encoded<T>(&self, encoded: String) -> Result<T, Error>
        where T: Store
    {        
        let key = format!("{}/{}", T::NAME, encoded);
        let path = self.path.clone().join(&key);

        // Acquire lock.
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();

        // Wait while key is blocked.
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
        // If key isn't locked, insert it into the locked set and release lock.
        guard.insert(key.clone(), Operation::Read(readers + 1));
        drop(guard);

        let output = (|| if !path.exists() {
            // Return error if file doesn't exist.
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
        } else {
            // Do the read
            Ok(T::deserialize(&mut fs::read(path)?)?)
        })();
        
        // Acquire lock again and decrease readers.
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
    pub fn read<T>(&self, id: &T::Id) -> Result<T, Error>
        where T: Store
    {        
        self.read_encoded(Database::encode(id)?)
    }

    /// Reads all entries from the database.
    pub fn read_all<T>(&self) -> Result<Vec<T>, Error>
        where T: Store
    {
        let mut result = Vec::new();
        match fs::read_dir(self.path.clone().join(T::NAME)) {
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
        where T: Store
    {
        let key = format!("{}/{}", T::NAME, Database::encode(object.id())?);
        let path = self.path.clone().join(&key);
        let mut directory = path.clone();
        directory.pop();

        // Acquire lock.
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();
        // Wait while key is blocked.
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // If key isn't locked, insert it into the locked set and release lock.
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if !path.exists() {
            // Return error if file doesn't exist.
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
        } else {
            // Do the update.
            if !directory.exists() {
                fs::create_dir_all(directory)?;
            }
            let mut file = OpenOptions::new().write(true).open(path)?;
            file.write_all(&object.serialize())?;
            file.flush()?;
            Ok(())
        })();

        // Acquire lock again and remove key from blocked list.
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    /// Tries to create an entry, updates it if it already exists.
    pub fn create_or_update<T>(&self, object: &T) -> Result<(), Error>
        where T: Store
    {
        if self.exists(object)? {
            self.update(object)?;
        } else {
            self.create(object)?;
        }

        Ok(())
    }

    fn delete_encoded<T>(&self, encoded: String) -> Result<(), Error>
        where T: Store
    {
        let key = format!("{}/{}", T::NAME, encoded);
        let path = self.path.clone().join(&key);

        // Acquire lock.
        let (lock, condvar) = &self.blocked;
        let mut guard = lock.lock().unwrap();
        // Wait while key is blocked.
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // If key isn't locked, insert it into the locked set and release lock.
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        let output = (|| if !path.exists() {
            // Return error if file doesn't exist.
            Err(Error::new(format!("Entry \"{}\" doesn't exist", key)))
        } else {
            // Do the delete.
            fs::remove_file(path)?;
            Ok(())
        })();

        // Acquire lock again and remove key from blocked list.
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        output
    }

    /// Deletes an entry from the database.
    pub fn delete<T>(&self, id: &T::Id) -> Result<(), Error>
        where T: Store
    {
        self.delete_encoded::<T>(Database::encode(id)?)
    }

    /// Delete all entries from the database
    pub fn delete_all<T>(&self) -> Result<(), Error>
        where T: Store
    {
        let mut result = Vec::new();
        match fs::read_dir(self.path.clone().join(T::NAME)) {
            Ok(paths) => for path in paths {
                let encoded = String::from(path?.path().into_iter().last().unwrap().to_str().unwrap());
                result.push(self.delete_encoded::<T>(encoded)?);
            },
            Err(_) => {}
        }

        Ok(())
    }

}