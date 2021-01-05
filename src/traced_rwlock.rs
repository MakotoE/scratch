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
        let mut trace = Trace {
            read_or_write: ReadOrWrite::Read,
            action: Action::Waiting,
            file,
            line,
        };
        if cfg!(debug_assertions) {
            self.recent_calls.borrow_mut().add(trace);
        }

        let result = self.inner.read().await;
        if cfg!(debug_assertions) {
            trace.action = Action::Acquired;
            self.recent_calls.borrow_mut().add(trace);
        }
        result
    }

    pub async fn write(&self, file: &'static str, line: u32) -> RwLockWriteGuard<'_, T> {
        let mut trace = Trace {
            read_or_write: ReadOrWrite::Write,
            action: Action::Waiting,
            file,
            line,
        };
        if cfg!(debug_assertions) {
            self.recent_calls.borrow_mut().add(trace);
        }

        let result = self.inner.write().await;
        if cfg!(debug_assertions) {
            trace.action = Action::Acquired;
            self.recent_calls.borrow_mut().add(trace);
        }
        result
    }

    #[allow(dead_code)]
    pub fn trace(&self) -> String {
        format!("{}", self.recent_calls.borrow())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Trace {
    read_or_write: ReadOrWrite,
    action: Action,
    file: &'static str,
    line: u32,
}

impl Default for Trace {
    fn default() -> Self {
        Self {
            read_or_write: ReadOrWrite::Read,
            action: Action::Waiting,
            file: "",
            line: 0,
        }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} at {}:{}",
            self.action, self.read_or_write, self.file, self.line
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum ReadOrWrite {
    Read,
    Write,
}

#[derive(Debug, Copy, Clone, PartialEq, strum::Display)]
enum Action {
    #[strum(serialize = "waiting for")]
    Waiting,
    Acquired,
}

const BUFFER_SIZE: usize = 64;

#[derive(Debug, Copy, Clone)]
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

impl Default for FixedSizeBuffer {
    fn default() -> Self {
        Self {
            buffer: [Trace::default(); BUFFER_SIZE],
            index: 0,
            size: 0,
        }
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
        f.write_str("[\n")?;
        for i in 0..usize::min(self.size, BUFFER_SIZE) {
            write!(f, "\t{},\n", &self[i]);
        }
        f.write_str("]")
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
                action: Action::Waiting,
                file: "file.rs",
                line: 1,
            });
            assert_eq!(format!("{}", buffer), "[ waiting for read at file.rs:1 ]");
        }
        {
            let mut buffer = FixedSizeBuffer::default();
            for i in 0..BUFFER_SIZE + 1 {
                buffer.add(Trace {
                    read_or_write: ReadOrWrite::Read,
                    action: Action::Waiting,
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

            let trace = Trace {
                read_or_write: ReadOrWrite::Write,
                action: Action::Waiting,
                file: "file.rs",
                line: 9,
            };
            buffer.add(trace);
            assert_eq!(buffer[BUFFER_SIZE - 1], trace);
        }
    }
}
