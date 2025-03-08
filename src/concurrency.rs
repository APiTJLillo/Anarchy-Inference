use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc::{self, Receiver, Sender};
use crate::error::LangError;

#[derive(Debug)]
pub struct Channel {
    sender: Mutex<Sender<Arc<Value>>>,
    receiver: Mutex<Receiver<Arc<Value>>>,
    buffer_size: usize,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Boolean(bool),
    Channel(Arc<Channel>),
    SharedState(Arc<SharedState>),
}

impl Channel {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);
        Channel {
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver),
            buffer_size,
        }
    }

    pub fn send(&self, value: Arc<Value>) -> Result<(), LangError> {
        let sender = self.sender.lock()
            .map_err(|_| LangError::runtime_error("Failed to acquire sender lock"))?;
        sender.try_send(value)
            .map_err(|_| LangError::runtime_error("Channel is full"))?;
        Ok(())
    }

    pub fn try_receive(&self) -> Result<Option<Arc<Value>>, LangError> {
        let mut receiver = self.receiver.try_lock()
            .map_err(|_| LangError::runtime_error("Failed to acquire receiver lock"))?;
        Ok(receiver.try_recv().ok())
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

#[derive(Debug)]
pub struct SharedState {
    values: RwLock<HashMap<String, Arc<Value>>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            values: RwLock::new(HashMap::new()),
        }
    }

    pub fn set(&self, key: String, value: Arc<Value>) -> Result<(), LangError> {
        let mut values = self.values.write()
            .map_err(|_| LangError::runtime_error("Failed to acquire write lock"))?;
        values.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Arc<Value>>, LangError> {
        let values = self.values.read()
            .map_err(|_| LangError::runtime_error("Failed to acquire read lock"))?;
        Ok(values.get(key).cloned())
    }
}

pub struct Scheduler {
    tasks: Mutex<Vec<Box<dyn FnOnce() -> Result<(), LangError> + Send>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Mutex::new(Vec::new()),
        }
    }

    pub fn schedule<F>(&self, task: F) -> Result<(), LangError>
    where
        F: FnOnce() -> Result<(), LangError> + Send + 'static,
    {
        let mut tasks = self.tasks.lock()
            .map_err(|_| LangError::runtime_error("Failed to acquire tasks lock"))?;
        tasks.push(Box::new(task));
        Ok(())
    }

    pub fn run_tasks(&self) -> Result<(), LangError> {
        let mut tasks = self.tasks.lock()
            .map_err(|_| LangError::runtime_error("Failed to acquire tasks lock"))?;
        while let Some(task) = tasks.pop() {
            task()?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scheduler {{ tasks: <{} tasks> }}", 
            self.tasks.try_lock().map(|t| t.len()).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_channel() {
        let channel = Channel::new(2);
        let value = Arc::new(Value::Number(42));
        
        assert!(channel.send(value.clone()).is_ok());
        let received = channel.try_receive().unwrap().unwrap();
        
        match &*received {
            Value::Number(n) => assert_eq!(*n, 42),
            _ => panic!("Expected number value"),
        }
    }

    #[tokio::test]
    async fn test_channel_buffer() {
        let channel = Channel::new(1);
        let value1 = Arc::new(Value::Number(1));
        let value2 = Arc::new(Value::Number(2));
        
        assert!(channel.send(value1).is_ok());
        assert!(channel.send(value2).is_err()); // Buffer is full
    }

    #[test]
    fn test_shared_state() {
        let state = SharedState::new();
        let value = Arc::new(Value::String("test".to_string()));
        
        assert!(state.set("key".to_string(), value.clone()).is_ok());
        let retrieved = state.get("key").unwrap().unwrap();
        
        match &*retrieved {
            Value::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new();
        let state = Arc::new(Mutex::new(0));
        let state_clone = state.clone();
        
        scheduler.schedule(move || {
            let mut value = state_clone.lock()
                .map_err(|_| LangError::runtime_error("Lock error"))?;
            *value += 1;
            Ok(())
        }).unwrap();
        
        scheduler.run_tasks().unwrap();
        
        assert_eq!(*state.lock().unwrap(), 1);
    }
} 