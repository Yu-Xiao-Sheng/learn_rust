use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, mpsc::{sync_channel, Receiver, SyncSender}};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;
use futures::task::{waker_ref, ArcWake};

struct TimerFuture {
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

// Task 是一个包装了 Future 的结构体，它实现了 ArcWake 特征。
struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    // wake_by_ref 方法会将任务放入任务队列中，通知执行器重新 poll 该任务。
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("任务队列已满");
    }
}

// Executor 是执行器，它负责从任务队列中取出任务并对其进行 poll
struct Executor {
    // 接受的的任务队列
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    // run 方法会不断从任务队列中取出任务，并对其进行 poll。如果 poll 返回 Poll::Pending，任务会被放回任务队列中，等待下一次 poll
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

// Spawner 提供了一个 spawn 方法，用于创建新的异步任务。
struct Spawner {
    // 任务队列
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    // spawn 方法接受一个实现了 Future 特征的任务。
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = Box::pin(future);
        // Spawner 将新创建的任务包装在一个 Task 结构体中，并发送到任务队列中，等待执行器调度和执行
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("任务队列已满");
    }
}

#[test]
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // 运行执行器
    executor.run();
}