mod storable;
mod serializable;

pub use storable::Storable;
pub use serializable::Serializable;
pub use storable_derive::Storable;
pub use serializable_derive::Serializable;
use std::fmt;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::Condvar;
use std::collections::HashMap;
// use std::any::Any;
// use std::time::SystemTime;

enum Operation {
    Write,
    Read(u32)
}

/*
type Pairs = HashMap<String, String>;

enum Method {
    Create(SystemTime, String, String, Pairs),
    Read(SystemTime, String, String),
    Update(SystemTime, String, String, Pairs),
    Delete(SystemTime, String, String)
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self {
            Method::Create(t, n, k, p) => write!(f, "{} create {} {} {:?}", t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), n, k, p),
            Method::Read(t, n, k) => write!(f, "{} read {} {}", t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), n, k),
            Method::Update(t, n, k, p) => write!(f, "{} update {} {} {:?}", t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), n, k, p),
            Method::Delete(t, n, k) => write!(f, "{} delete {} {}", t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos(), n, k)
        }
    }
}
*/

pub struct Database {
    blocked: (Mutex<HashMap<String, Operation>>, Condvar)
    // cache: Mutex<(HashMap<String, Box<dyn Any + Send>>, Vec<String>)>,
    // log: Mutex<File>
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
    fn description(&self) -> &str {
        "description"
    }
}

impl Database {
    pub fn new() -> Database {
        /*
        let directory_string = "data";
        let directory = Path::new(directory_string);
        if !directory.exists() {
            fs::create_dir_all(directory).expect("Data directory could not be created");
        }
        */
        Database {
            blocked: (Mutex::new(HashMap::new()), Condvar::new())
            // cache: Mutex::new((HashMap::new(), Vec::new()))
            /*,
            log: Mutex::new(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open("data/log.txt")
                    .expect("Could not open log file")
            )*/
        }
    }

    pub fn create<T: Storable + Serializable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
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

        // return error if file exists
        if path.exists() {
            return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
        }

        // do the create
        // self.append_log(Method::Create(SystemTime::now(), T::name(), object.id(), HashMap::new()));
        let directory_string = format!("data/{}", T::name()?);
        let directory = Path::new(&directory_string);
        if !directory.exists() {
            fs::create_dir_all(directory)?;
        }
        let mut file = File::create(path)?;
        file.write_all(&object.serialize())?;
        file.flush()?;
        drop(file);

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn read<T: Storable + Serializable>(&self, object: &mut T) -> Result<(), Box<dyn std::error::Error>> {
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

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
        }

        // do the read
        // self.append_log(Method::Read(SystemTime::now(), T::name(), object.id()));
        /*
        let guard = self.cache.lock().unwrap();
        let (map, list) = (&guard.0, &guard.1);
        match map.get(&key) {
            Some(o) => {
                object = o.downcast_mut::<T>().unwrap();
                list.remove(&key);
                list.push(key.clone());
            }
            None => {
                map.insert(key.clone(), Box::new(*object));
                list.push(key.clone());
            }
        }
        */
        object.deserialize(&mut fs::read(path)?);

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
            guard.insert(key.clone(), Operation::Read(updated_readers));
        }
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn update<T: Storable + Serializable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
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

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
        }

        // do the update
        // self.append_log(Method::Update(SystemTime::now(), T::name(), object.id(), HashMap::new()));
        let directory_string = format!("data/{}", T::name()?);
        let directory = Path::new(&directory_string);
        if !directory.exists() {
            fs::create_dir_all(directory)?;
        }
        let mut file = OpenOptions::new().write(true).open(path)?;
        file.write_all(&object.serialize())?;
        file.flush()?;
        drop(file);

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    pub fn delete<T: Storable + Serializable>(&self, object: &T) -> Result<(), Box<dyn std::error::Error>> {
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

        // return error if file doesn't exist
        if !path.exists() {
            return Err(Box::new(Error::new()) as Box<dyn std::error::Error>);
        }

        // do the delete
        // self.append_log(Method::Delete(SystemTime::now(), T::name(), object.id()));
        fs::remove_file(path)?;

        // acquire lock again and remove key from blocked list
        let mut guard = lock.lock().unwrap();
        guard.remove(&key);
        drop(guard);
        condvar.notify_all();

        Ok(())
    }

    /*
    fn append_log(&self, method: Method) {
        writeln!(self.log.lock().unwrap(), "{}", method).unwrap();
    }
    */
}