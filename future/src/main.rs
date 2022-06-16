#![feature(get_mut_unchecked)]

use std::{
    future::Future,
    pin::Pin,
    ptr::{read_volatile, write_volatile},
    sync::Arc,
    task::{Context, Poll},
    thread::{sleep, spawn},
    time::Duration,
};

use futures::{executor::block_on, task::AtomicWaker};

struct RecvFuture {
    state: Arc<State>,
}

type Msg = Option<Box<[u8]>>;

struct State {
    done: bool,
    msg: Msg,
    waker: AtomicWaker,
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

impl Future for RecvFuture {
    type Output = Msg;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = &self.state;
        let done = unsafe { read_volatile(&state.done as _) };
        if done {
            let state = unsafe { Arc::get_mut_unchecked(&mut self.get_mut().state) };
            //let mut msg = unsafe { read_volatile(&state.msg as *const Msg) }.take();
            Poll::Ready(state.msg.take())
        } else {
            state.waker.register(cx.waker());
            Poll::Pending
        }
    }
}

async fn recv() -> Option<Box<[u8]>> {
    let future = RecvFuture::new();
    let mut state = future.state.clone();
    spawn(move || {
        sleep(Duration::from_secs(1));
        #[allow(unused_mut)]
        let mut state = unsafe { Arc::get_mut_unchecked(&mut state) };
        unsafe {
            write_volatile(&mut state.msg as *mut Msg, Some(Box::new([1, 2, 3])));
            write_volatile(&mut state.done as _, true);
        }
        state.waker.wake();
    });
    /*
    let state2 = future.state.clone();
    spawn(move || {
        sleep(Duration::from_secs(2));
        dbg!("wake again");
        state2.waker.wake();
    });
    */
    future.await
}

fn main() {
    loop {
        println!("begin");
        dbg!(block_on(recv()));
        sleep(Duration::from_secs(3));
        println!("end");
    }
}
