#![allow(dead_code)]
#![allow(unused_imports)]

use rusqlite::{Connection, params};
use std::sync::mpsc::{self, Sender, Receiver};

pub enum Error {

}

pub type OperationResult = std::result::Result<(), Error>;
pub type DatabaseOperation = Box<dyn Fn(Transaction) -> OperationResult + Send>;

pub struct Transaction<'conn> {
    conn: &'conn Connection,
}

impl<'conn> Transaction<'conn> {
    pub fn get_value(&self, id: i64) -> rusqlite::Result<String> {
        let r: String = self.conn.query_row("SELECT value FROM config WHERE id = ?1", params![id], |row| {
            row.get(0)
        })?;
        Ok(r)
    }

    pub fn save_value(&self, id: i64, value: &str) -> rusqlite::Result<usize> {
        self.conn.execute("INSERT INTO config (id, value) VALUES (?1, ?2)", params![id, value])
    }
}

pub struct Db {
    sender: std::sync::mpsc::Sender<DatabaseOperation>,
    tx_queue: std::thread::JoinHandle<()>,
}

impl Db {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<DatabaseOperation>();
        let thread_builder = std::thread::Builder::new().name("tx-queue".into());
        let tx_queue = thread_builder.spawn(move || {
            let conn = Connection::open_in_memory().unwrap();
            Self::create_schema(&conn);

            while let Ok(op) = receiver.recv() {
                let _ = conn.execute_batch("BEGIN TRANSACTION");
                let tx = Transaction{conn: &conn};
                match op(tx) {
                    Ok(_) => {
                        let _ = conn.execute_batch("COMMIT");
                    }
                    Err(_) => {
                        let _ = conn.execute_batch("ROLLBACK");
                    }
                }
            }
            let _ = conn.close();
        }).unwrap();

        Self {
            sender,
            tx_queue,
        }
    }

    pub fn create_schema(conn: &Connection) {
        if let Err(e) = conn.execute_batch("CREATE TABLE IF NOT EXISTS config (id INTEGER PRIMARY KEY, value TEXT)") {
            println!("ERROR: {}", e);
        }
    }

    pub fn perform(&self, op: DatabaseOperation) {
        self.sender.send(op).unwrap();
    }

    pub fn wait_to_complete(self) {
        self.tx_queue.join().unwrap();
    }
}