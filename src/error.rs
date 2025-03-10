use std::fmt;
use std::error::Error;
use std::io;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub location: SourceLocation,
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at {} ({})", self.function, self.location)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    SyntaxError,
    TypeError,
    RuntimeError,
    IOError,
    NetworkError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::SyntaxError => write!(f, "Syntax Error"),
            ErrorKind::TypeError => write!(f, "Type Error"),
            ErrorKind::RuntimeError => write!(f, "Runtime Error"),
            ErrorKind::IOError => write!(f, "IO Error"),
            ErrorKind::NetworkError => write!(f, "Network Error"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Syntax,
    Runtime,
    Type,
    IO,
    Semantic,
}

#[derive(Debug, Clone)]
pub struct LangError {
    pub error_type: ErrorType,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub stack_trace: Vec<StackFrame>,
}

impl LangError {
    pub fn syntax_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::Syntax,
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn syntax_error_with_location(message: &str, line: usize, column: usize) -> Self {
        LangError {
            error_type: ErrorType::Syntax,
            message: message.to_string(),
            location: Some(SourceLocation {
                line,
                column,
                file: String::new(),
            }),
            stack_trace: Vec::new(),
        }
    }

    pub fn runtime_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::Runtime,
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn runtime_error_with_location(message: &str, location: SourceLocation) -> Self {
        LangError {
            error_type: ErrorType::Runtime,
            message: message.to_string(),
            location: Some(location),
            stack_trace: Vec::new(),
        }
    }

    pub fn type_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::Type,
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn type_error_with_location(message: &str, location: SourceLocation) -> Self {
        LangError {
            error_type: ErrorType::Type,
            message: message.to_string(),
            location: Some(location),
            stack_trace: Vec::new(),
        }
    }

    pub fn io_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::IO,
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn semantic_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::Semantic,
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn semantic_error_with_location(message: &str, location: SourceLocation) -> Self {
        LangError {
            error_type: ErrorType::Semantic,
            message: message.to_string(),
            location: Some(location),
            stack_trace: Vec::new(),
        }
    }

    pub fn network_error(message: &str) -> Self {
        LangError {
            error_type: ErrorType::IO,  // Using IO type for network errors
            message: message.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }

    pub fn with_stack_trace(mut self, stack_trace: Vec<StackFrame>) -> Self {
        self.stack_trace = stack_trace;
        self
    }
}

impl fmt::Display for LangError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error_type = match self.error_type {
            ErrorType::Syntax => "Syntax",
            ErrorType::Runtime => "Runtime",
            ErrorType::Type => "Type",
            ErrorType::IO => "IO",
            ErrorType::Semantic => "Semantic",
        };

        if let Some(location) = &self.location {
            write!(f, "{} error at {}:{}:{}: {}", 
                error_type,
                location.file,
                location.line,
                location.column,
                self.message
            )?;
        } else {
            write!(f, "{} error: {}", error_type, self.message)?;
        }

        if !self.stack_trace.is_empty() {
            writeln!(f, "\nStack trace:")?;
            for frame in &self.stack_trace {
                writeln!(f, "  at {} ({}:{}:{})",
                    frame.function,
                    frame.location.file,
                    frame.location.line,
                    frame.location.column
                )?;
            }
        }

        Ok(())
    }
}

impl Error for LangError {}

impl From<Box<dyn Error>> for LangError {
    fn from(error: Box<dyn Error>) -> Self {
        LangError {
            error_type: ErrorType::Runtime,
            message: error.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }
}

impl From<io::Error> for LangError {
    fn from(error: io::Error) -> Self {
        LangError {
            error_type: ErrorType::IO,
            message: error.to_string(),
            location: None,
            stack_trace: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_error() {
        let error = LangError::syntax_error("Unexpected token");
        assert!(matches!(error.error_type, ErrorType::Syntax));
        assert_eq!(error.message, "Unexpected token");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_syntax_error_with_location() {
        let error = LangError::syntax_error_with_location("Unexpected token", 1, 5);
        assert!(matches!(error.error_type, ErrorType::Syntax));
        assert_eq!(error.message, "Unexpected token");
        assert!(error.location.is_some());
        let location = error.location.unwrap();
        assert_eq!(location.line, 1);
        assert_eq!(location.column, 5);
    }

    #[test]
    fn test_runtime_error() {
        let error = LangError::runtime_error("Division by zero");
        assert!(matches!(error.error_type, ErrorType::Runtime));
        assert_eq!(error.message, "Division by zero");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_type_error() {
        let error = LangError::type_error("Expected number, found string");
        assert!(matches!(error.error_type, ErrorType::Type));
        assert_eq!(error.message, "Expected number, found string");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_semantic_error() {
        let error = LangError::semantic_error("Undefined variable");
        assert!(matches!(error.error_type, ErrorType::Semantic));
        assert_eq!(error.message, "Undefined variable");
        assert!(error.location.is_none());
    }

    #[test]
    fn test_error_with_stack_trace() {
        let location = SourceLocation {
            line: 1,
            column: 5,
            file: "test.ai".to_string(),
        };
        let frame = StackFrame {
            function: "main".to_string(),
            location: location.clone(),
        };
        let error = LangError::runtime_error("Test error")
            .with_stack_trace(vec![frame]);
        assert_eq!(error.stack_trace.len(), 1);
        assert_eq!(error.stack_trace[0].function, "main");
    }
}
