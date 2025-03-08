use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use crate::concurrency::{Channel, SharedState};
use crate::ui::UI;
use crate::ast::ASTNode;
use crate::interpreter::Environment;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(i64),
    String(String),
    Boolean(bool),
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<ASTNode>,
        closure: Arc<Environment>
    },
    Connection(Arc<TcpStream>),
    Channel(Arc<Channel>),
    SharedState(Arc<SharedState>),
    UI(Arc<UI>),
    Void,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Object(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                    first = false;
                }
                write!(f, "}}")
            },
            Value::Array(items) => {
                write!(f, "[")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                    first = false;
                }
                write!(f, "]")
            },
            Value::Function { name, .. } => write!(f, "<function {}>", name),
            Value::Connection(_) => write!(f, "<connection>"),
            Value::Channel(_) => write!(f, "<channel>"),
            Value::SharedState(_) => write!(f, "<shared state>"),
            Value::UI(_) => write!(f, "<ui>"),
            Value::Void => write!(f, "void"),
        }
    }
}
