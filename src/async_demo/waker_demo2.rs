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
                // 重点：这里执行wake方法，实际上是通过内部的
                // 一个虚函数表执行了Task实现的ArcWake特征的wake_by_ref方法
                // 该方法重新将任务发送到任务队列,等待下次被执行。
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output=()> + Send>>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        println!("wake_by_ref called"); // 添加日志，显示 wake_by_ref 被调用
        arc_self.task_sender.send(cloned).expect("任务队列已满");
    }
}

struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
    fn run(&self) {
        // 接收任务(阻塞的接受任务)
        while let Ok(task) = self.ready_queue.recv() {
            println!("task executed"); // 添加日志，显示任务被执行
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // 通过Arc<impl ArcWake>的引用创建一个Waker的引用类型,内部会初始化一个虚函数表
                // 在运行时可以执行真正实现的ArcWake特征
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if future.as_mut().poll(context).is_pending() {
                    *future_slot = Some(future);
                }
            }
        }
    }
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    // 创建一个Channel，多生产者，单消费者通道。
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    // spawn方法用将Future发送到任务队列中
    fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
        // Box::pin让Future在内存中的位置固定
        let future = Box::pin(future);
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
    // 在 Rust 中，当你使用 async 块时，编译器会将其转换为一个实现了 Future 特征的状态机。这个状态机会在每次调用 poll 方法时推进到下一个状态，直到完成。
    spawner.spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // 运行执行器
    thread::spawn(move || {
        executor.run();
    });

    // 等待足够长的时间以确保任务能够完成
    thread::sleep(Duration::new(3, 0));
}