use std::thread;

fn main() {
    let operations: [Box<dyn Fn(&str) + Send>; 2] = [Box::new(|param| {
        println!("first closure({}) {:?}", param, thread::current().name());
    }), Box::new(|param| {
        println!("second closure({}) {:?}", param, thread::current().name());
    })];

    println!("thread: {:?}", thread::current().name());

    let thread_builder = thread::Builder::new().name("operation-queue".into());
    let handle = thread_builder.spawn(move || {
        for op in &operations {
            op("hello");
        }
    }).unwrap();

    handle.join().unwrap();
}
