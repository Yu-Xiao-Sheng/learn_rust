use std::{sync::{Arc, Mutex}, thread};

#[derive(Debug)]
#[warn(dead_code)]
struct MyBox(*const u8);
unsafe impl Send for MyBox {}
unsafe impl Sync for MyBox {}

#[test]
pub fn test_send_and_sync() {
    let b = &MyBox(5 as *const u8);
    let v = Arc::new(Mutex::new(b));
    let t = thread::spawn(move || {
        let _v1 =  v.lock().unwrap();
    });

    t.join().unwrap();
}