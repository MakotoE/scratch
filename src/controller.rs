use super::*;
use tokio::sync::Notify;

#[derive(Debug)]
pub struct ThreadController {
    semphore: Notify,
    state: RwLock<PauseState>,
}

impl ThreadController {
    pub fn new() -> Self {
        Self {
            semphore: Notify::new(),
            state: RwLock::new(PauseState::Continue),
        }
    }

    pub async fn wait(&self) {
        if *self.state.read().await == PauseState::Paused {
            self.semphore.notified().await;
        }
    }

    pub async fn continue_(&self) {
        *self.state.write().await = PauseState::Continue;
        self.semphore.notify();
        log::info!("continuing");
    }

    pub async fn pause(&self) {
        *self.state.write().await = PauseState::Paused;
        log::info!("paused");
    }

    pub fn step(&self) {
        self.semphore.notify();
        log::info!("step");
    }

    pub async fn state(&self) -> PauseState {
        *self.state.read().await
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum PauseState {
    Continue,
    Paused,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn thread_controller() {
        let controller = ThreadController::new();
        assert_eq!(controller.state().await, PauseState::Continue);
        controller.wait().await;
        controller.pause().await;
        assert_eq!(controller.state().await, PauseState::Paused);
        controller.step();
        controller.wait().await;
        controller.continue_().await;
        assert_eq!(controller.state().await, PauseState::Continue);
        controller.wait().await;
    }
}
