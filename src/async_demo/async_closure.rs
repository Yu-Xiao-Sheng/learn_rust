use std::future::Future;

async fn task<F, Fut>(process: F, list: Vec<String>)
where
    F: Fn(Vec<String>) -> Fut,
    Fut: Future<Output=Vec<String>> + Send,
{
    println!("原始数据:{:?}", list);
    let result = process(list).await;
    println!("处理后的数据:{:?}", result);
}

#[tokio::test]
async fn main() {
    let list = vec![String::from("a"), String::from("b"), String::from("c")];
    task(|list| async move {
        list.into_iter().map(|s| s.to_uppercase()).collect()
    }, list).await
}