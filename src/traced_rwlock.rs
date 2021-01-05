use super::*;
use std::collections::VecDeque;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct TracedRwLock<T> {
    inner: RwLock<T>,
    recent_access: RefCell<RecentAccess>,
}

impl<T> TracedRwLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RwLock::new(inner),
            recent_access: RefCell::new(RecentAccess::Read(VecDeque::new())),
        }
    }

    pub async fn read(&self, file: &'static str, line: u32) -> RwLockReadGuard<'_, T> {
        let result = self.inner.read().await;
        if cfg!(debug_assertions) {
            self.recent_access
                .borrow_mut()
                .add_read(Trace { file, line });
        }
        result
    }

    pub async fn write(&self, file: &'static str, line: u32) -> RwLockWriteGuard<'_, T> {
        let result = self.inner.write().await;
        if cfg!(debug_assertions) {
            self.recent_access
                .borrow_mut()
                .add_write(Trace { file, line });
        }
        result
    }

    #[allow(dead_code)]
    pub fn trace(&self) -> String {
        format!("{}", self.recent_access.borrow())
    }
}

#[derive(Debug, Clone, PartialEq)]
enum RecentAccess {
    Read(VecDeque<Trace>),
    Write(Trace),
}

impl RecentAccess {
    fn add_read(&mut self, trace: Trace) {
        match self {
            RecentAccess::Read(r) => {
                r.push_back(trace);
                if r.len() > 8 {
                    r.pop_front();
                }
            }
            RecentAccess::Write(_) => *self = RecentAccess::Read(vec![trace].into()),
        }
    }

    fn add_write(&mut self, trace: Trace) {
        match self {
            RecentAccess::Read(_) => *self = RecentAccess::Write(trace),
            RecentAccess::Write(w) => *w = trace,
        }
    }
}

impl Display for RecentAccess {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RecentAccess::Read(reads) => {
                f.write_str("RwLock was last read: [\n")?;
                for trace in reads {
                    write!(f, "\t{},\n", trace)?;
                }
                f.write_str("]")
            }
            RecentAccess::Write(write) => {
                f.write_str("RwLock was last written: ")?;
                write!(f, "{}", write)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Trace {
    file: &'static str,
    line: u32,
}

impl Default for Trace {
    fn default() -> Self {
        Self { file: "", line: 0 }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.file, self.line)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_recent_access() {
        let mut recent_access = RecentAccess::Read(VecDeque::new());
        assert_eq!(format!("{}", recent_access), "RwLock was last read: [\n]");

        recent_access.add_read(Trace {
            file: "file.rs",
            line: 1,
        });
        assert_eq!(
            format!("{}", recent_access),
            "RwLock was last read: [\n\tfile.rs:1,\n]"
        );

        recent_access.add_write(Trace {
            file: "file.rs",
            line: 1,
        });
        assert_eq!(
            format!("{}", recent_access),
            "RwLock was last written: file.rs:1"
        );
    }
}
