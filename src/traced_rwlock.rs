use super::*;
use std::ops::Index;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct TracedRwLock<T> {
    inner: RwLock<T>,
    recent_calls: RefCell<FixedSizeBuffer>,
}

impl<T> TracedRwLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RwLock::new(inner),
            recent_calls: RefCell::new(FixedSizeBuffer::default()),
        }
    }

    pub async fn read(&self, file: &'static str, line: u32) -> RwLockReadGuard<'_, T> {
        if cfg!(debug_assertions) {
            self.recent_calls.borrow_mut().add(Trace {
                read_or_write: ReadOrWrite::Read,
                file,
                line,
            });
        }
        self.inner.read().await
    }

    pub async fn write(&self, file: &'static str, line: u32) -> RwLockWriteGuard<'_, T> {
        if cfg!(debug_assertions) {
            self.recent_calls.borrow_mut().add(Trace {
                read_or_write: ReadOrWrite::Write,
                file,
                line,
            });
        }
        self.inner.write().await
    }

    pub fn trace(&self) -> String {
        format!("{}", self.recent_calls.borrow())
    }
}

#[derive(Debug, Copy, Clone)]
struct Trace {
    read_or_write: ReadOrWrite,
    file: &'static str,
    line: u32,
}

impl Default for Trace {
    fn default() -> Self {
        Self {
            read_or_write: ReadOrWrite::Read,
            file: "",
            line: 0,
        }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}:{}", self.read_or_write, self.file, self.line)
    }
}

#[derive(Debug, Copy, Clone)]
enum ReadOrWrite {
    Read,
    Write,
}

impl Display for ReadOrWrite {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReadOrWrite::Read => "read",
            ReadOrWrite::Write => "write",
        })
    }
}

const BUFFER_SIZE: usize = 8;

#[derive(Debug, Copy, Clone, Default)]
struct FixedSizeBuffer {
    buffer: [Trace; BUFFER_SIZE],
    index: usize,
    size: usize,
}

impl FixedSizeBuffer {
    fn add(&mut self, trace: Trace) {
        self.buffer[self.index] = trace;
        self.index = (self.index + 1) % BUFFER_SIZE;
        self.size += 1;
    }
}

impl Index<usize> for FixedSizeBuffer {
    type Output = Trace;

    fn index(&self, index: usize) -> &Trace {
        if index >= usize::min(self.size, BUFFER_SIZE) {
            panic!("index is out of range");
        }

        let buffer_index = if self.size < BUFFER_SIZE {
            index
        } else {
            (self.index + index) % BUFFER_SIZE
        };
        &self.buffer[buffer_index]
    }
}

impl Display for FixedSizeBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[ ")?;
        for i in 0..usize::min(self.size, BUFFER_SIZE) {
            Display::fmt(&self[i], f)?;

            if i < usize::min(self.size, BUFFER_SIZE) - 1 {
                f.write_str(", ")?;
            }
        }
        f.write_str(" ]")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::panic;

    #[test]
    fn test_fixed_size_buffer() {
        {
            let buffer = FixedSizeBuffer::default();
            assert_eq!(format!("{}", buffer), "[  ]");
        }
        {
            let mut buffer = FixedSizeBuffer::default();
            buffer.add(Trace {
                read_or_write: ReadOrWrite::Read,
                file: "file.rs",
                line: 1,
            });
            assert_eq!(format!("{}", buffer), "[ read at file.rs:1 ]");
        }
        {
            let mut buffer = FixedSizeBuffer::default();
            for i in 0..BUFFER_SIZE + 1 {
                buffer.add(Trace {
                    read_or_write: ReadOrWrite::Read,
                    file: "file.rs",
                    line: i as u32,
                });
            }
            assert_eq!(buffer[0].line, 1);
            assert_eq!(buffer[BUFFER_SIZE - 1].line, 8);

            let prev_hook = panic::take_hook();
            panic::set_hook(Box::new(|_| {}));
            assert!(panic::catch_unwind(|| buffer[BUFFER_SIZE]).is_err());
            panic::set_hook(Box::new(prev_hook));
        }
    }
}
