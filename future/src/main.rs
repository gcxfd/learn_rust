#![feature(get_mut_unchecked)]

use std::{
    future::Future,
    pin::Pin,
    ptr,
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
        state.waker.register(cx.waker());
        let done = unsafe { ptr::read_volatile(&state.done as _) };
        if done {
            //let state = unsafe { Arc::get_mut_unchecked(&mut self.get_mut().state) };
            let msg = unsafe { ptr::read_volatile(&state.msg as *const Msg) };
            Poll::Ready(msg)
        } else {
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
            ptr::write_volatile(&mut state.msg as *mut Msg, Some(Box::new([1, 2, 3])));
            ptr::write_volatile(&mut state.done as _, true);
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
    println!("begin");
    dbg!(block_on(recv()));
    sleep(Duration::from_secs(3));
    println!("end");
}
