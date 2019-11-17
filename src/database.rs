use super::storable::Storable;
use std::fmt;
use std::path::Path;
use std::fs;
use std::fs::File;
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

        // return error if file exists
        if path.exists() {
            return Err(Box::new(Error::new()));
        }

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

    pub fn update<T: Storable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
        let key = object.key();
        let path_string = format!("data/{}.bin", &key);
        let path = Path::new(&path_string);

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()));
        }

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

        // do the update
        let mut file = File::open(path)?;
        file.write_all(b"It worked again!")?;
        drop(file);

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }
}