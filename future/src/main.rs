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
        let done = unsafe { std::ptr::read_volatile(&state.done as _) };
        if done {
            let state = unsafe { Arc::get_mut_unchecked(&mut self.get_mut().state) };
            Poll::Ready(state.msg.take())
        } else {
            Poll::Pending
        }
    }
}

impl RecvFuture {
    pub fn new() -> Self {
        let state = Arc::new(State {
            done: false,
            waker: AtomicWaker::new(),
            msg: None,
        });

        RecvFuture { state }
    }
}

async fn recv() -> Option<Box<[u8]>> {
    let future = RecvFuture::new();
    let mut state = future.state.clone();
    spawn(move || {
        sleep(Duration::from_secs(1));
        let mut state = unsafe { Arc::get_mut_unchecked(&mut state) };
        state.msg = Some(Box::new([1, 2, 3]));

        unsafe {
            std::ptr::write_volatile(&mut state.done as _, true);
        }
        state.waker.wake();
    });
    future.await
}

fn main() {
    println!("begin");
    dbg!(block_on(recv()));
    println!("end");
}
