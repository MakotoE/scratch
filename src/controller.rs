use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug)]
pub struct DebugController {
    semphore: Arc<ControllerSemaphore>,
    display_debug: tokio::sync::RwLock<bool>,
    interval_id: tokio::sync::RwLock<i32>,
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            semphore: Arc::new(ControllerSemaphore::new()),
            display_debug: tokio::sync::RwLock::new(false),
            interval_id: tokio::sync::RwLock::new(0),
        }
    }

    pub async fn wait(&self) {
        self.semphore.acquire().await;
    }

    pub async fn continue_(&self, _speed: Speed) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(*self.interval_id.read().await);
        self.semphore.reset().await;
        self.semphore.set_blocking(true).await;
        *self.display_debug.write().await = false;

        let semaphore = self.semphore.clone();
        let cb = Closure::wrap(Box::new(move || {
            semaphore.add_permit();
        }) as Box<dyn Fn()>);
        *self.interval_id.write().await = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(&cb.as_ref().unchecked_ref(), 1)
            .unwrap();
        cb.forget();

        log::info!("continuing");
    }

    pub async fn pause(&self) {
        web_sys::window()
            .unwrap()
            .clear_interval_with_handle(*self.interval_id.read().await);
        self.semphore.reset().await;
        self.semphore.set_blocking(true).await;
        *self.display_debug.write().await = true;

        log::info!("paused");
    }

    pub fn step(&self) {
        self.semphore.add_permit();

        log::info!("step");
    }

    pub async fn display_debug(&self) -> bool {
        *self.display_debug.read().await
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Speed {
    Normal,
}

#[derive(Debug)]
struct ControllerSemaphore {
    semaphore: tokio::sync::Semaphore,
    blocking: tokio::sync::RwLock<bool>,
}

impl ControllerSemaphore {
    fn new() -> Self {
        Self {
            semaphore: tokio::sync::Semaphore::new(0),
            blocking: tokio::sync::RwLock::new(false),
        }
    }

    async fn acquire(&self) {
        if *self.blocking.read().await {
            self.semaphore.acquire().await.forget();
        }
    }

    fn add_permit(&self) {
        self.semaphore.add_permits(1);
    }

    async fn set_blocking(&self, blocking: bool) {
        *self.blocking.write().await = blocking;

        if !blocking {
            self.add_permit();
        }
    }

    async fn reset(&self) {
        while self.semaphore.available_permits() > 0 {
            match self.semaphore.try_acquire() {
                Ok(p) => p.forget(),
                Err(_) => break,
            }
        }
        *self.blocking.write().await = false;
    }

    #[allow(dead_code)]
    async fn available(&self) -> bool {
        self.semaphore.available_permits() > 0 || !*self.blocking.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn controller_semaphore() {
        let semaphore = ControllerSemaphore::new();
        assert!(semaphore.available().await);
        semaphore.set_blocking(true).await;
        assert!(!semaphore.available().await);
        semaphore.add_permit();
        assert!(semaphore.available().await);
        semaphore.acquire().await;
        assert!(!semaphore.available().await);
        semaphore.reset().await;
        assert!(semaphore.available().await);
    }
}
