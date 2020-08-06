#![allow(dead_code)]
#![allow(unused_imports)]

use rusqlite::{params, Connection};
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum Error {
    Save,
    GetValue,
}

#[derive(Debug)]
pub enum ResponseType {
    String(String),
}

pub type OperationResult<T> = std::result::Result<T, Error>;
pub type DatabaseOperation = Box<dyn Fn(Transaction) -> OperationResult<ResponseType> + Send>;

pub struct Transaction<'conn> {
    conn: &'conn Connection,
}

impl<'conn> Transaction<'conn> {
    pub fn get_value(&self, id: i64) -> OperationResult<ResponseType> {
        let out = self
            .conn
            .query_row(
                "SELECT value FROM config WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| Error::GetValue)?;

        Ok(ResponseType::String(out))
    }

    pub fn save_value(&self, id: i64, value: &str) -> OperationResult<()> {
        self.conn
            .execute(
                "INSERT INTO config (id, value) VALUES (?1, ?2)",
                params![id, value],
            )
            .map_err(|_| Error::Save)
            .map(|_| ())
    }
}

struct Operation {
    name: &'static str,
    block: DatabaseOperation,
    result_passback: Sender<OperationResult<ResponseType>>,
}

#[derive(Debug)]
pub struct Db {
    sender: std::sync::mpsc::Sender<Operation>,
    tx_queue: std::thread::JoinHandle<()>,
}

impl Db {
    pub fn new() -> Self {
        let (sender, receiver) =
            mpsc::channel::<Operation>();

        let thread_builder = std::thread::Builder::new().name("tx-queue".into());
        let tx_queue = thread_builder
            .spawn(move || {
                let conn = Connection::open_in_memory().unwrap();
                Self::create_schema(&conn);

                while let Ok(op) = receiver.recv() {
                    let _ = conn.execute_batch("BEGIN TRANSACTION");
                    let tx = Transaction { conn: &conn };
                    println!("Running transaction '{}' on thread {}", op.name, std::thread::current().name().unwrap());
                    match (op.block)(tx) {
                        Ok(resp) => {
                            let _ = conn.execute_batch("COMMIT");
                            op.result_passback.send(Ok(resp)).unwrap();

                        }
                        Err(e) => {
                            let _ = conn.execute_batch("ROLLBACK");
                            println!("Transaction {} failed, rolling back!", op.name);
                            op.result_passback.send(Err(e)).unwrap();
                        }
                    }
                }
                let _ = conn.close();
            })
            .unwrap();

        Self { sender, tx_queue }
    }

    fn create_schema(conn: &Connection) {
        if let Err(e) = conn
            .execute_batch("CREATE TABLE IF NOT EXISTS config (id INTEGER PRIMARY KEY, value TEXT)")
        {
            println!("ERROR: {}", e);
        }
    }

    pub fn transaction(&self, name: &'static str, op: DatabaseOperation) -> OperationResult<ResponseType> {
        let (passback, rx) = std::sync::mpsc::channel();
        let op = Operation {
            block: op,
            name,
            result_passback: passback,
        };

        self.sender.send(op).unwrap();
        rx.recv().unwrap()
    }

    pub fn wait_to_complete(self) {
        self.tx_queue.join().unwrap();
    }
}
