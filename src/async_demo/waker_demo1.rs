use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // 注册 waker
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // 启动一个新线程，模拟异步计时器
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                // 触发 waker，通知执行器
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

#[test]
fn main() {
    let timer_future = TimerFuture::new(Duration::new(2, 0));
    let waker = futures::task::noop_waker();
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(timer_future);

    // 执行器首次调用 poll
    match future.as_mut().poll(&mut context) {
        Poll::Ready(_) => println!("Task completed"),
        Poll::Pending => println!("Task is pending"),
    }

    // 等待计时器完成
    thread::sleep(Duration::new(3, 0));
    // match future.as_mut().poll(&mut context) {
    //     Poll::Ready(_) => println!("Task completed"),
    //     Poll::Pending => println!("Task is pending"),
    // }
}