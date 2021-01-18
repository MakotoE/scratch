use super::*;
use std::collections::VecDeque;
use std::ops;
use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct TracedRwLock<T> {
    inner: RwLock<T>,
    recent_access: RefCell<RecentAccess>,
    lock_owner: Rc<RefCell<Option<RecentAccess>>>,
}

impl<T> TracedRwLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: RwLock::new(inner),
            recent_access: RefCell::new(RecentAccess::Read(VecDeque::new())),
            lock_owner: Rc::new(RefCell::new(None)),
        }
    }

    pub async fn read(&self, file: &'static str, line: u32) -> TracedRwLockReadGuard<'_, T> {
        let result = self.inner.read().await;
        self.lock_owner
            .borrow_mut()
            .get_or_insert(RecentAccess::Read(VecDeque::new()))
            .add_read(Trace { file, line });
        TracedRwLockReadGuard {
            inner: result,
            lock_owner: self.lock_owner.clone(),
        }
    }

    pub async fn write(&self, file: &'static str, line: u32) -> TracedRwLockWriteGuard<'_, T> {
        let result = self.inner.write().await;
        self.lock_owner
            .borrow_mut()
            .get_or_insert(RecentAccess::Write(Trace { file, line }));
        TracedRwLockWriteGuard {
            inner: result,
            lock_owner: self.lock_owner.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn trace(&self) -> String {
        format!("{}", self.recent_access.borrow())
    }
}

#[derive(Debug)]
pub struct TracedRwLockReadGuard<'a, T: ?Sized> {
    inner: RwLockReadGuard<'a, T>,
    lock_owner: Rc<RefCell<Option<RecentAccess>>>,
}

impl<T: ?Sized> Drop for TracedRwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        *self.lock_owner.borrow_mut() = None;
    }
}

impl<T: ?Sized> ops::Deref for TracedRwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

#[derive(Debug)]
pub struct TracedRwLockWriteGuard<'a, T: ?Sized> {
    inner: RwLockWriteGuard<'a, T>,
    lock_owner: Rc<RefCell<Option<RecentAccess>>>,
}

impl<T: ?Sized> Drop for TracedRwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        *self.lock_owner.borrow_mut() = None;
    }
}

impl<T: ?Sized> ops::Deref for TracedRwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T: ?Sized> ops::DerefMut for TracedRwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
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
    }
}
