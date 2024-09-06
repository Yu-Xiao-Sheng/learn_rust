use std::time::Duration;
use futures::executor::block_on;

async fn foo() {
    println!("foo start");
    //1. rust的异步方法调用之后，不使用.await或者不阻塞或者不被poll就不会被执行，以下这种方式调用将不会执行bar方法
    // bar()
    //2. 将bar()异步方法交给一个运行时，tokio::spawn 用于创建一个异步任务，它将在异步运行时中被poll，然后会被执行
    tokio::spawn(bar());
    //3. 直接在当前异步方法中，等待另一个异步方法执行完成后再执行。他需要当前函数也是异步函数才行。
    bar().await;
    println!("foo end");
}

async fn bar() {
    println!("bar start");
    tokio::time::sleep(Duration::from_secs(1)).await; // 暂停 1 秒
    println!("bar end");
}

#[tokio::test]
async fn main() {
    foo().await;
}

#[tokio::test]
async fn main2() {
    block_on(foo());
    tokio::time::sleep(Duration::from_secs(3)).await; // 暂停 1 秒
}