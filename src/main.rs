use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    task::{Context, Poll},
    thread::{sleep, spawn},
    time::Duration,
};

use futures::{executor::block_on, task::AtomicWaker};

struct TimerFuture {
    shared_state: Arc<SharedState>,
}

/// Future和Thread共享的数据
struct SharedState {
    completed: AtomicBool,
    waker: AtomicWaker,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 调用register更新Waker，再读取共享的completed变量.
        self.shared_state.waker.register(cx.waker());
        if self.shared_state.completed.load(SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(SharedState {
            completed: AtomicBool::new(false),
            waker: AtomicWaker::new(),
        });

        let thread_shared_state = shared_state.clone();
        spawn(move || {
            sleep(duration);
            thread_shared_state.completed.store(true, SeqCst);
            thread_shared_state.waker.wake();
        });

        TimerFuture { shared_state }
    }
}

fn main() {
    block_on(TimerFuture::new(Duration::from_secs(3)));
    println!("Hello, world!");
}
