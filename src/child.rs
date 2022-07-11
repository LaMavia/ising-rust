use std::thread::{self, spawn, JoinHandle};

#[derive(Debug)]
pub struct Child {
    pub name: String,
    pub msg: String,
    pub thread_handle: Box<JoinHandle<()>>,
    pub done: bool,
}

impl Child {
    pub fn make<F: FnOnce() -> () + Send + 'static>(name: &String, f: F) -> Self {
        Child {
            name: name.to_owned(),
            msg: "<starting>".to_string(),
            thread_handle: Box::new(thread::Builder::new().name(name.to_owned()).spawn(f).unwrap()),
            done: false,
        }
    }

    pub fn update(&mut self, msg: &ChildMsg) {
        if self.name == msg.name {
            self.msg = msg.msg.clone();
            self.done = msg.done;
        }
    }
}

#[derive(Debug)]
pub struct ChildMsg {
    pub name: String,
    pub msg: String,
    pub done: bool,
}

impl ChildMsg {
    pub fn make(name: String, msg: String, done: bool) -> Self {
        ChildMsg { name, msg, done }
    }
}

#[macro_export]
macro_rules! send {
    ($tx:expr, $name:expr, $ex:expr) => {
        $tx.send(ChildMsg::make($name.to_owned(), $ex, false))
            .unwrap();
    };

    (final $tx:expr, $name:expr, $ex:expr) => {
        $tx.send(ChildMsg::make($name.to_owned(), $ex, true))
            .unwrap();
    };
}

pub(crate) use send;
