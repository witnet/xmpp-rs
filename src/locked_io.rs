use std::io;
use std::io::prelude::*;

use std::sync::{Arc, Mutex};

pub struct LockedIO<T>(Arc<Mutex<T>>);

impl<T> LockedIO<T> {
    pub fn from(inner: Arc<Mutex<T>>) -> LockedIO<T> {
        LockedIO(inner)
    }
}

impl<T> Clone for LockedIO<T> {
    fn clone(&self) -> LockedIO<T> {
        LockedIO(self.0.clone())
    }
}

impl<T: Write> io::Write for LockedIO<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.flush()
    }
}

impl<T: Read> io::Read for LockedIO<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner = self.0.lock().unwrap(); // TODO: make safer
        inner.read(buf)
    }
}
