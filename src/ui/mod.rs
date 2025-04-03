use std::sync::Mutex;
use std::error::Error;
use crate::error::LangError;

pub mod components;

#[derive(Debug)]
pub struct UI {
    active_windows: Mutex<Vec<String>>,
}

impl UI {
    pub fn new() -> Self {
        UI {
            active_windows: Mutex::new(Vec::new()),
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn create_window(&self, title: String, _width: f64, _height: f64) -> Result<(), LangError> {
        let mut windows = self.active_windows.lock().unwrap();
        windows.push(title);
        Ok(())
    }

    pub fn add_text(&self, _text: String) -> Result<(), LangError> {
        // In a real implementation, this would add text to the current window
        Ok(())
    }
}

pub use components::*;

#[derive(Debug)]
pub struct YewApp;

impl YewApp {
    pub fn new() -> Self {
        Self
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
