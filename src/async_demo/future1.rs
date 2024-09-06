use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct PureComputation {
    count: usize,
}

impl Future for PureComputation {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // 模拟一些计算
        for _ in 0..this.count {
            // 这里可以进行一些计算操作
        }

        // 任务完成，返回结果
        Poll::Ready(this.count)
    }
}

#[test]
fn main() {
    let computation = PureComputation { count: 1000000000 };
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(computation);

    match future.as_mut().poll(&mut context) {
        Poll::Ready(result) => println!("Computation completed with result: {}", result),
        Poll::Pending => println!("Computation is still pending"),
    }
}