use std::thread;
mod database;
use database::Db;

fn main() {
    // Db::transaction(move |tx: Transaction| {
    //     tx.get_accounts();
    //     tx.save_vault().map_err();
    //     Ok(())
    //     return Err()
    // });
    println!("thread: {:?}", thread::current().name());

    let db = Db::new();
    db.perform(Box::new(|tx| {
        tx.save_value(1, "value 1");
        println!("transaction #1 ({}) {:?}", tx.get_value(1), thread::current().name());
    }));

    db.perform(Box::new(|tx| {
        tx.save_value(2, "value 2");
        println!("transaction #2 ({}) {:?}", tx.get_value(2), thread::current().name());
    }));


    db.wait_to_complete();
}
