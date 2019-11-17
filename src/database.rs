use super::storable::Storable;
use std::fmt;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::HashMap;

enum Operation {
    Write,
    Read(u32)
}

pub struct Database {
    blocked: Arc<(Mutex<HashMap<String, Operation>>, Condvar)>
}

#[derive(Debug)]
pub struct Error {

}

impl Error {
    pub fn new() -> Error {
        Error {
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl std::error::Error for Error {

}

impl Database {
    pub fn new() -> Database {
        Database {
            blocked: Arc::new((Mutex::new(HashMap::new()), Condvar::new()))
        }
    }

    pub fn create<T: Storable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
        let key = object.key();
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &*self.blocked;
        let mut guard = lock.lock().unwrap();
        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        // return error if file exists
        if path.exists() {
            return Err(Box::new(Error::new()));
        }

        // do the save
        let directory_string = format!("data/{}", T::name());
        let directory = Path::new(&directory_string);
        if !directory.exists() {
            fs::create_dir_all(directory)?;
        }
        let mut file = File::create(path)?;
        file.write_all(b"It worked!")?;
        drop(file);

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn read<T: Storable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
        let key = object.key();
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &*self.blocked;
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

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()));
        }

        // do the read
        let file = File::open(path)?;

        // acquire lock again and decrease readers
        let mut guard = lock.lock().unwrap();
        let updated_readers = match (*guard).get(&key) {
            Some(operation) => match operation {
                Operation::Read(r) => *r,
                Operation::Write => panic!("This should never happen")
            },
            None => panic!("This should never happen")
        } - 1;
        if updated_readers == 0 {
            guard.remove(&key);
        } else {
            guard.insert(key.clone(), Operation::Read(updated_readers));
        }
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn update<T: Storable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
        let key = object.key();
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &*self.blocked;
        let mut guard = lock.lock().unwrap();
        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()));
        }

        // do the update
        let mut file = OpenOptions::new().write(true).open(path)?;
        file.write_all(b"It worked again!")?;
        drop(file);

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn delete<T: Storable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
        let key = object.key();
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // acquire lock
        let (lock, condvar) = &*self.blocked;
        let mut guard = lock.lock().unwrap();
        // wait while key is blocked
        while (*guard).get(&key).is_some() {
            guard = condvar.wait(guard).unwrap();
        }
        // if key isn't locked, insert it into the locked set and release lock
        guard.insert(key.clone(), Operation::Write);
        drop(guard);

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()));
        }

        // do the delete
        fs::remove_file(path)?;

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }
}