use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    thread::{sleep, spawn},
    time::Duration,
};

use futures::executor::block_on;

struct Task {
    val: u32,
}

impl Future for Task {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        dbg!("pool");
        if self.val == 0 {
            Poll::Pending
        } else {
            Poll::Ready(self.val)
        }
    }
}

async fn test() -> u32 {
    let mut task = Task { val: 0 };
    let f = task.await;
    spawn(move || {
        sleep(Duration::from_secs(1));
        dbg!("sleeped");
        task.val = 3;
    });
    f
}

fn main() {
    dbg!(block_on(test()));
    println!("Hello, world!");
}
