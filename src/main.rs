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
    state: Arc<State>,
}

/// Future和Thread共享的数据
struct State {
    completed: AtomicBool,
    msg: Option<Box<[u8]>>,
    waker: AtomicWaker,
}

impl Future for TimerFuture {
    type Output = Option<Box<[u8]>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 调用register更新Waker，再读取共享的completed变量.
        let state = &self.state;
        state.waker.register(cx.waker());
        if state.completed.load(SeqCst) {
            let state = Arc::get_mut(&mut self.get_mut().state).unwrap();
            Poll::Ready(state.msg.take())
        } else {
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let state = Arc::new(State {
            completed: AtomicBool::new(false),
            waker: AtomicWaker::new(),
            msg: None,
        });

        let thread_state = state.clone();
        spawn(move || {
            sleep(duration);
            thread_state.msg = Some(Box::new([1, 2, 3]));
            thread_state.completed.store(true, SeqCst);
            thread_state.waker.wake();
        });

        TimerFuture { state }
    }
}

fn main() {
    dbg!(block_on(TimerFuture::new(Duration::from_secs(3))));
    println!("Hello, world!");
}
