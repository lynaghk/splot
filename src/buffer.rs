use std::sync::Arc;
use tokio::sync::Notify;

/// Ring buffer with monotonic index
pub struct R<T> {
    /// number of items to store
    n: usize,
    top: usize,
    v: Vec<T>, // TODO: replace with mut slice so consumers can provide their own buffer?
    notify: Arc<Notify>,
}

#[derive(Debug)]
pub enum GetResult<T> {
    Ok(T),
    Expired,
    WaitUntil(Arc<Notify>),
}

impl<T: Clone> R<T> {
    // TODO: get rid of this default param and use MaybeUninit for the unobservable initial vector data
    pub fn new(default: T, n: usize) -> Self {
        Self {
            n,
            top: 0,
            v: std::iter::repeat(default).take(n).collect(),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn get(&self, idx: usize) -> GetResult<T> {
        use GetResult::*;
        if idx < self.bottom() {
            return Expired;
        }

        if idx < self.top() {
            return Ok(self.v[idx % self.n].clone());
        }

        return WaitUntil(self.notify.clone());
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn bottom(&self) -> usize {
        self.top.saturating_sub(self.n)
    }

    pub fn push(&mut self, x: T) {
        self.v[self.top % self.n] = x;
        self.top += 1;
        self.notify.notify_waiters();
    }
}
