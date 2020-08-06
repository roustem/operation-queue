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

    let value_in = "Some value #1".to_string();
    let res = db.transaction("first thing", Box::new(move |tx| {
        tx.save_value(1, &value_in).unwrap();
        tx.get_value(1)
    }));

    println!("transaction #1 ({:?}) result on thread '{:?}'", res.unwrap(), thread::current().name());

    let res_2 = db.transaction("second thing", Box::new(|tx| {
        tx.save_value(2, "value too awesome").unwrap();
        tx.get_value(2)
    }));

    println!("transaction #2 ({:?}) result on thread '{:?}'", res_2.unwrap(), thread::current().name());

    db.wait_to_complete();
}
