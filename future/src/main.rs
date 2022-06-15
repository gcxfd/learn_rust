#![feature(get_mut_unchecked)]

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    thread::{sleep, spawn},
    time::Duration,
};

use futures::{executor::block_on, task::AtomicWaker};

struct RecvFuture {
    state: Arc<State>,
}

struct State {
    done: bool,
    msg: Option<Box<[u8]>>,
    waker: AtomicWaker,
}

impl Future for RecvFuture {
    type Output = Option<Box<[u8]>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = &self.state;
        state.waker.register(cx.waker());
        if state.done {
            let state = unsafe { Arc::get_mut_unchecked(&mut self.get_mut().state) };
            Poll::Ready(state.msg.take())
        } else {
            Poll::Pending
        }
    }
}

impl RecvFuture {
    pub fn new(duration: Duration) -> Self {
        let state = Arc::new(State {
            done: false,
            waker: AtomicWaker::new(),
            msg: None,
        });

        let mut thread_state = state.clone();
        spawn(move || {
            sleep(duration);
            let mut thread_state = unsafe { Arc::get_mut_unchecked(&mut thread_state) };
            thread_state.done = true;
            thread_state.msg = Some(Box::new([1, 2, 3]));
            thread_state.waker.wake();
        });

        RecvFuture { state }
    }
}

fn main() {
    println!("begin");
    dbg!(block_on(RecvFuture::new(Duration::from_secs(1))));
    println!("end");
}
