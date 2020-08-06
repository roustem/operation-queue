#![allow(dead_code)]
#![allow(unused_imports)]

use rusqlite::{Connection};
use std::sync::mpsc::{self, Sender, Receiver};

// pub enum Error {
//
// }

pub type DatabaseOperation = Box<dyn Fn(Transaction) + Send>;
// pub type OperationResult = std::result::Result<(), Error>;

pub struct Transaction {

}

impl Transaction {
    pub fn get_value(&self, id: i64) -> String {
        format!("{}", id)
    }

    pub fn save_value(&self, _id: i64, _value: &str) {

    }
}

pub struct Db {
    conn: Connection,
    sender: std::sync::mpsc::Sender<DatabaseOperation>,
    tx_queue: std::thread::JoinHandle<()>,
}

impl Db {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();

        let (sender, receiver) = mpsc::channel::<DatabaseOperation>();
        let thread_builder = std::thread::Builder::new().name("tx-queue".into());
        let tx_queue = thread_builder.spawn(move || {
            loop {
                while let Ok(op) = receiver.recv() {
                    let tx = Transaction{};
                    op(tx);
                }
            }
        }).unwrap();

        Self {
            conn,
            sender,
            tx_queue,
        }
    }

    pub fn perform(&self, op: DatabaseOperation) {
        self.sender.send(op).unwrap();
    }

    pub fn wait_to_complete(self) {
        self.tx_queue.join().unwrap();
    }
}