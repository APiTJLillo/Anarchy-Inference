// Interpreter for the minimal LLM-friendly language

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::ast::{ASTNode, NodeType};
use crate::error::{LangError, SourceLocation, StackFrame};
use crate::lexer::Token;
use crate::network::Network;
use crate::concurrency::{Channel, SharedState, Scheduler};
use crate::ui::UI;
use crate::value::Value;
use log::debug;

impl Value {
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Array(items) => {
                if items.len() == 1 {
                    items[0].as_number()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::Array(items) => {
                if items.len() == 1 {
                    items[0].as_string()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            Value::Array(items) => {
                if items.len() == 1 {
                    items[0].as_bool()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl From<Value> for crate::concurrency::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(n) => crate::concurrency::Value::Number(n),
            Value::String(s) => crate::concurrency::Value::String(s),
            Value::Boolean(b) => crate::concurrency::Value::Boolean(b),
            Value::Channel(c) => crate::concurrency::Value::Channel(c),
            Value::SharedState(s) => crate::concurrency::Value::SharedState(s),
            Value::Array(items) => {
                let sum: i64 = items.iter().filter_map(|v| {
                    if let Value::Number(n) = v {
                        Some(*n)
                    } else {
                        None
                    }
                }).sum();
                crate::concurrency::Value::Number(sum)
            }
            _ => crate::concurrency::Value::Number(0),
        }
    }
}

impl From<crate::concurrency::Value> for Value {
    fn from(value: crate::concurrency::Value) -> Self {
        match value {
            crate::concurrency::Value::Number(n) => Value::Number(n),
            crate::concurrency::Value::String(s) => Value::String(s),
            crate::concurrency::Value::Boolean(b) => Value::Boolean(b),
            crate::concurrency::Value::Channel(c) => Value::Channel(c),
            crate::concurrency::Value::SharedState(s) => Value::SharedState(s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Arc<Environment>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), LangError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            Err(LangError::runtime_error(&format!(
                "Cannot assign to variable '{}' in parent scope",
                name
            )))
        } else {
            Err(LangError::runtime_error(&format!("Undefined variable '{}'", name)))
        }
    }
}

pub struct Interpreter {
    environment: Environment,
    functions: HashMap<String, (Vec<String>, Vec<ASTNode>)>,
    libraries: HashMap<String, HashMap<String, (Vec<String>, Vec<ASTNode>)>>,
    current_library: Option<String>,
    network: Arc<Network>,
    scheduler: Arc<Scheduler>,
    shared_state: Arc<RwLock<HashMap<String, Value>>>,
    current_file: String,
    current_function: Option<String>,
    stack_trace: Vec<StackFrame>,
    current_node: Option<ASTNode>,
    ui: Arc<UI>,
    current_window: Option<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            functions: HashMap::new(),
            libraries: HashMap::new(),
            current_library: None,
            network: Arc::new(Network::new()),
            scheduler: Arc::new(Scheduler::new()),
            shared_state: Arc::new(RwLock::new(HashMap::new())),
            current_file: String::new(),
            current_function: None,
            stack_trace: Vec::new(),
            current_node: None,
            ui: Arc::new(UI::new()),
            current_window: None,
        };

        // Initialize UI library first
        interpreter.init_ui_library();

        // Define UI object in environment
        interpreter.environment.define("⬢".to_string(), Value::UI(interpreter.ui.clone()));

        interpreter
    }

    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }

    pub fn interpret(&mut self, ast: &[ASTNode]) -> Result<Value, LangError> {
        let mut last_value = Value::Number(0);
        for node in ast {
            self.current_node = Some(node.clone());
            last_value = self.interpret_node(node)?;
            
            // If this is a UI start call, we need to block here
            if let Value::Void = last_value {
                if let Some(node) = &self.current_node {
                    if let NodeType::MethodCall { object: _, method, arguments: _ } = &node.node_type {
                        if method == "start" {
                            // This will block until the window is closed
                            let ui = self.ui.clone();
                            return ui.start()
                                .map_err(|e| LangError::runtime_error(&format!("Failed to start UI: {}", e)))
                                .map(|_| Value::Void);
                        }
                    }
                }
            }
        }
        Ok(last_value)
    }

    fn push_stack_frame(&mut self, function: String, location: SourceLocation) {
        self.stack_trace.push(StackFrame { function, location });
    }

    fn pop_stack_frame(&mut self) {
        self.stack_trace.pop();
    }

    fn interpret_node(&mut self, node: &ASTNode) -> Result<Value, LangError> {
        debug!("Interpreting node: {:?}", node.node_type);
        match &node.node_type {
            NodeType::Number(n) => {
                debug!("Interpreting Number node: {}", n);
                Ok(Value::Number(*n))
            },
            NodeType::String(s) => {
                debug!("Interpreting String node: {}", s);
                Ok(Value::String(s.clone()))
            },
            NodeType::Boolean(b) => {
                debug!("Interpreting Boolean node: {}", b);
                Ok(Value::Boolean(*b))
            },
            NodeType::Identifier(name) => {
                debug!("Interpreting Identifier node: {}", name);
                if name == "⬢" {
                    // Special handling for UI object
                    if let Some(value) = self.environment.get("⬢") {
                        Ok(value)
                    } else {
                        Err(LangError::runtime_error("UI object not found in environment"))
                    }
                } else if let Some(value) = self.environment.get(name) {
                    Ok(value)
                } else {
                    Err(LangError::runtime_error(&format!("Undefined identifier: {}", name)))
                }
            },
            NodeType::PropertyAccess { object, property } => {
                debug!("Interpreting PropertyAccess node: object={:?}, property={}", object.node_type, property);
                let obj = self.interpret_node(object)?;
                debug!("Property access object value: {:?}", obj);
                match &obj {
                    Value::Object(ref map) => {
                        if let Some(value) = map.get(property) {
                            debug!("Found property {} in object", property);
                            match value {
                                Value::Function { name, parameters, body, closure } => {
                                    // Create a new function with the object bound to it
                                    let mut new_env = Environment::with_parent(closure.clone());
                                    new_env.define("self".to_string(), obj.clone());
                                    Ok(Value::Function {
                                        name: name.clone(),
                                        parameters: parameters.clone(),
                                        body: body.clone(),
                                        closure: Arc::new(new_env),
                                    })
                                },
                                _ => Ok(value.clone())
                            }
                        } else {
                            debug!("Property {} not found in object", property);
                            Err(LangError::runtime_error(&format!("Property {} not found", property)))
                        }
                    },
                    Value::UI(_) => {
                        debug!("Creating function for UI method: {}", property);
                        // For UI objects, create a function that can be called
                        let name = property.clone();
                        let (parameters, arguments) = match name.as_str() {
                            "start" => (vec![], vec![]),
                            "□" | "create_window" => {
                                let params = vec!["title".to_string(), "width".to_string(), "height".to_string()];
                                let args = params.iter().map(|p| ASTNode {
                                    node_type: NodeType::Identifier(p.clone()),
                                    line: 0,
                                    column: 0
                                }).collect();
                                (params, args)
                            },
                            "⬚" | "add_text" => {
                                let params = vec!["text".to_string()];
                                let args = params.iter().map(|p| ASTNode {
                                    node_type: NodeType::Identifier(p.clone()),
                                    line: 0,
                                    column: 0
                                }).collect();
                                (params, args)
                            },
                            _ => return Err(LangError::runtime_error(&format!("Unknown method: {}", name)))
                        };
                        Ok(Value::Function {
                            name: name.clone(),
                            parameters,
                            body: Box::new(ASTNode {
                                node_type: NodeType::MethodCall {
                                    object: Box::new(ASTNode {
                                        node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                                        line: 0,
                                        column: 0
                                    }),
                                    method: name,
                                    arguments
                                },
                                line: 0,
                                column: 0
                            }),
                            closure: Arc::new(Environment::new())
                        })
                    },
                    _ => {
                        debug!("Cannot access properties on type: {:?}", obj);
                        Err(LangError::runtime_error("Cannot access properties on this type"))
                    }
                }
            },
            NodeType::SymbolicKeyword(name) => {
                debug!("Interpreting SymbolicKeyword node: {}", name);
                if let Some(value) = self.environment.get(name) {
                    Ok(value)
                    } else {
                    Err(LangError::runtime_error(&format!("Undefined symbolic keyword: {}", name)))
                }
            },
            NodeType::Library { name, functions } => {
                debug!("Interpreting Library node: {}", name);
                let mut lib_obj = HashMap::new();
                
                // If this is the UI library, initialize it
                if name == "ui" {
                    let ui = Arc::new(UI::new());
                    lib_obj.insert("⬢".to_string(), Value::UI(ui.clone()));
                    self.ui = ui;

                    // Define symbolic keywords in the library object
                    lib_obj.insert("□".to_string(), Value::Function {
                        name: "create_window".to_string(),
                        parameters: vec!["title".to_string(), "width".to_string(), "height".to_string()],
                        body: Box::new(ASTNode {
                            node_type: NodeType::MethodCall {
                                object: Box::new(ASTNode {
                                    node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                                    line: 0,
                                    column: 0
                                }),
                                method: "create_window".to_string(),
                                arguments: vec![
                                    ASTNode {
                                        node_type: NodeType::Identifier("title".to_string()),
                                        line: 0,
                                        column: 0
                                    },
                                    ASTNode {
                                        node_type: NodeType::Identifier("width".to_string()),
                                        line: 0,
                                        column: 0
                                    },
                                    ASTNode {
                                        node_type: NodeType::Identifier("height".to_string()),
                                        line: 0,
                                        column: 0
                                    }
                                ]
                            },
                            line: 0,
                            column: 0
                        }),
                        closure: Arc::new(Environment::new())
                    });

                    lib_obj.insert("⬚".to_string(), Value::Function {
                        name: "add_text".to_string(),
                        parameters: vec!["text".to_string()],
                        body: Box::new(ASTNode {
                            node_type: NodeType::MethodCall {
                                object: Box::new(ASTNode {
                                    node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                                    line: 0,
                                    column: 0
                                }),
                                method: "add_text".to_string(),
                                arguments: vec![
                                    ASTNode {
                                        node_type: NodeType::Identifier("text".to_string()),
                                        line: 0,
                                        column: 0
                                    }
                                ]
                            },
                            line: 0,
                            column: 0
                        }),
                        closure: Arc::new(Environment::new())
                    });
                }

                // Create a new environment for the library
                let mut lib_env = Environment::new();
                for (key, value) in lib_obj.iter() {
                    lib_env.define(key.clone(), value.clone());
                }

                // Interpret each function in the library
                for function in functions {
                    if let NodeType::FunctionDeclaration { name: func_name, parameters, body } = &function.node_type {
                        lib_obj.insert(func_name.clone(), Value::Function {
                            name: func_name.clone(),
                            parameters: parameters.clone(),
                            body: body.clone(),
                            closure: Arc::new(lib_env.clone())
                        });
                    }
                }

                // Create namespace object and define it in the environment
                let namespace = Value::Object(lib_obj);
                self.environment.define(name.to_string(), namespace.clone());
                Ok(namespace)
            },
            NodeType::Binary { left, operator, right } => {
                debug!("Interpreting Binary node: {:?}", operator);
                let left_val = self.interpret_node(left)?;
                let right_val = self.interpret_node(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            },
            NodeType::Unary { operator, operand } => {
                debug!("Interpreting Unary node: {:?}", operator);
                let operand_val = self.interpret_node(operand)?;
                self.evaluate_unary_op(operator, &operand_val)
            },
            NodeType::Assignment { name, value } => {
                debug!("Interpreting Assignment node: {}", name);
                let value_val = self.interpret_node(value)?;
                // Define the variable in the environment instead of trying to assign
                self.environment.define(name.clone(), value_val.clone());
                Ok(value_val)
            },
            NodeType::Block(statements) => {
                debug!("Interpreting Block node with {} statements", statements.len());
                let mut last_value = Value::Null;
                for stmt in statements {
                    last_value = self.interpret_node(stmt)?;
                }
                Ok(last_value)
            },
            NodeType::FunctionDeclaration { name, parameters, body } => {
                debug!("Interpreting FunctionDeclaration node: {}", name);
                let func = Value::Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                    closure: Arc::new(self.environment.clone()),
                };

                // Check if we're inside a library
                if let Some(lib_name) = &self.current_library {
                    if let Some(Value::Object(mut obj)) = self.environment.get(lib_name) {
                                    let mut new_obj = obj.clone();
                                    new_obj.insert(name.clone(), func.clone());
                        self.environment.define(lib_name.clone(), Value::Object(new_obj));
                                    return Ok(func);
                    }
                }

                // If not in a library, define in current environment
                self.environment.define(name.clone(), func.clone());
                Ok(func)
            },
            NodeType::FunctionCall { callee, arguments } => {
                debug!("Interpreting FunctionCall node");
                let callee_val = match &callee.node_type {
                    NodeType::SymbolicKeyword(name) => {
                                if let Some(value) = self.environment.get(name) {
                            Ok(value)
                                } else {
                            Err(LangError::runtime_error(&format!("Undefined symbolic keyword: {}", name)))
                        }
                    },
                    _ => self.interpret_node(callee),
                }?;

                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.interpret_node(arg)?);
                }

                match callee_val {
                    Value::Function { name, parameters, body, closure } => {
                        if arguments.len() != parameters.len() {
                            return Err(LangError::runtime_error(&format!(
                                "Wrong number of arguments for function {}: expected {}, got {}",
                                name, parameters.len(), arguments.len()
                            )));
                        }

                        let mut new_env = Environment::with_parent(closure.clone());
                        for (param, arg) in parameters.iter().zip(arg_values) {
                            new_env.define(param.clone(), arg);
                        }

                        let prev_env = std::mem::replace(&mut self.environment, new_env);
                        let result = self.interpret_node(&body);
                        self.environment = prev_env;
                        result
                    },
                    _ => Err(LangError::runtime_error("Cannot call non-function value")),
                }
            },
            NodeType::Channel(size_expr) => {
                let size = match self.interpret_node(size_expr)? {
                    Value::Number(n) => n as usize,
                    _ => return Err(LangError::type_error("Channel size must be a number")),
                };
                Ok(Value::Channel(Arc::new(Channel::new(size))))
            },
            NodeType::Send { channel, value } => {
                let channel_val = self.interpret_node(channel)?;
                let value_val = self.interpret_node(value)?;
                if let Value::Channel(channel) = channel_val {
                    channel.send(Arc::new(value_val.clone().into()))?;
                    Ok(Value::Null)
                } else {
                    Err(LangError::type_error("Expected channel"))
                }
            },
            NodeType::Receive(channel) => {
                let channel_val = self.interpret_node(channel)?;
                if let Value::Channel(channel) = channel_val {
                    match channel.try_receive()? {
                        Some(value) => Ok((*value).clone().into()),
                        None => Ok(Value::Null),
                    }
                } else {
                    Err(LangError::type_error("Expected channel"))
                }
            },
            NodeType::SharedState { name, value } => {
                let value = self.interpret_node(value)?;
                self.shared_state.write().unwrap().insert(name.to_string(), value.clone());
                Ok(value)
            },
            NodeType::SetSharedState { name, value } => {
                let value = self.interpret_node(value)?;
                if let Some(state) = self.shared_state.write().unwrap().get_mut(&name.to_string()) {
                    *state = value.clone();
                    Ok(value)
                } else {
                    Err(LangError::runtime_error(&format!("Shared state '{}' not found", name)))
                }
            },
            NodeType::GetSharedState { name } => {
                if let Some(value) = self.shared_state.read().unwrap().get(&name.to_string()) {
                    Ok(value.clone())
                } else {
                    Err(LangError::runtime_error(&format!("Shared state '{}' not found", name)))
                }
            },
            NodeType::Lambda { params, body } => {
                Ok(Value::Function {
                    name: "lambda".to_string(),
                    parameters: params.clone(),
                    body: body.clone(),
                    closure: Arc::new(self.environment.clone()),
                })
            },
            NodeType::Variable(name) => {
                debug!("Interpreting Variable node: {}", name);
                        if let Some(value) = self.environment.get(name) {
                    Ok(value)
                        } else {
                    Err(LangError::runtime_error(&format!("Undefined variable: {}", name)))
                }
            },
            NodeType::MethodCall { object, method, arguments } => {
                let obj = self.interpret_node(object)?;
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.interpret_node(arg)?);
                }
                self.call_method(obj, method, arg_values)
            },
            other => {
                debug!("Encountered unsupported node type: {:?}", other);
                Err(LangError::runtime_error(&format!("Unsupported node type: {:?}", other)))
            }
        }
    }

    fn evaluate_binary_op(&self, left: &Value, operator: &Token, right: &Value) -> Result<Value, LangError> {
        match (left, operator, right) {
            (Value::Number(l), Token::SymbolicOperator('+'), Value::Number(r)) => Ok(Value::Number(l + r)),
            (Value::Number(l), Token::SymbolicOperator('-'), Value::Number(r)) => Ok(Value::Number(l - r)),
            (Value::Number(l), Token::SymbolicOperator('*'), Value::Number(r)) => Ok(Value::Number(l * r)),
            (Value::Number(l), Token::SymbolicOperator('/'), Value::Number(r)) => {
                if *r == 0 {
                    Err(LangError::runtime_error("Division by zero"))
                } else {
                    Ok(Value::Number(l / r))
                }
            },
            (Value::String(l), Token::SymbolicOperator('+'), Value::String(r)) => Ok(Value::String(l.clone() + r)),
            (Value::String(l), Token::SymbolicOperator('='), Value::String(r)) => Ok(Value::Boolean(l == r)),
            (Value::Number(l), Token::SymbolicOperator('='), Value::Number(r)) => Ok(Value::Boolean(l == r)),
            (Value::Boolean(l), Token::SymbolicOperator('='), Value::Boolean(r)) => Ok(Value::Boolean(l == r)),
            _ => Err(LangError::type_error(&format!("Invalid binary operation: {:?} {} {:?}", left, operator, right))),
        }
    }

    fn evaluate_unary_op(&self, operator: &Token, operand: &Value) -> Result<Value, LangError> {
        match (operator, operand) {
            (Token::SymbolicOperator('-'), Value::Number(n)) => Ok(Value::Number(-n)),
            (Token::SymbolicOperator('!'), Value::Boolean(b)) => Ok(Value::Boolean(!b)),
            _ => Err(LangError::type_error(&format!("Invalid unary operation: {} {:?}", operator, operand))),
        }
    }

    pub fn init_ui_library(&mut self) {
        let ui = Arc::new(UI::new());
        self.ui = ui.clone();

        let mut env = Environment::new();
        env.define("⬢".to_string(), Value::UI(ui.clone()));

        // Create window function
        env.define("create_window".to_string(), Value::Function {
            name: "create_window".to_string(),
            parameters: vec!["title".to_string(), "width".to_string(), "height".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::MethodCall {
                    object: Box::new(ASTNode {
                        node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                line: 0,
                column: 0,
            }),
                    method: "create_window".to_string(),
                    arguments: vec![
                    ASTNode {
                            node_type: NodeType::Identifier("title".to_string()),
                        line: 0,
                        column: 0,
                        },
                        ASTNode {
                            node_type: NodeType::Identifier("width".to_string()),
                line: 0,
                column: 0,
                        },
                    ASTNode {
                            node_type: NodeType::Identifier("height".to_string()),
                        line: 0,
                        column: 0,
                        },
                    ],
                },
                line: 0,
                column: 0,
            }),
            closure: Arc::new(env.clone()),
        });

        // Add text function
        env.define("add_text".to_string(), Value::Function {
            name: "add_text".to_string(),
            parameters: vec!["text".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::MethodCall {
                    object: Box::new(ASTNode {
                        node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                        line: 0,
                        column: 0,
                    }),
                    method: "add_text".to_string(),
                    arguments: vec![
                    ASTNode {
                            node_type: NodeType::Identifier("text".to_string()),
                        line: 0,
                        column: 0,
                        },
                    ],
                },
                line: 0,
                column: 0,
            }),
            closure: Arc::new(env.clone()),
        });

        // Map symbolic functions to their implementations
        env.define("□".to_string(), env.get("create_window").unwrap());
        env.define("⬚".to_string(), env.get("add_text").unwrap());
        env.define("✎".to_string(), env.get("add_text").unwrap()); // Text is same as ⬚
        env.define("⌨".to_string(), Value::Function {
            name: "add_input".to_string(),
            parameters: vec!["placeholder".to_string(), "callback".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::MethodCall {
                    object: Box::new(ASTNode {
                        node_type: NodeType::SymbolicKeyword("⬢".to_string()),
                        line: 0,
                        column: 0
                    }),
                    method: "add_input".to_string(),
                    arguments: vec![
                        ASTNode {
                            node_type: NodeType::Identifier("placeholder".to_string()),
                            line: 0,
                            column: 0
                        },
                        ASTNode {
                            node_type: NodeType::Identifier("callback".to_string()),
                            line: 0,
                            column: 0
                        }
                    ]
                },
                line: 0,
                column: 0
            }),
            closure: Arc::new(env.clone())
        });

        self.environment = env;
    }

    fn init_core_library(&mut self) {
        let mut core_obj = HashMap::new();
        core_obj.insert("⌽".to_string(), Value::Function {
            name: "⌽".to_string(),
            parameters: vec!["text".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });
        self.environment.define("∇".to_string(), Value::Object(core_obj.clone()));
    }

    fn init_network_library(&mut self) {
        let mut net_obj = HashMap::new();
        
        // Define network methods
        net_obj.insert("⊲".to_string(), Value::Function {
            name: "⊲".to_string(),
            parameters: vec!["port".to_string(), "handler".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        net_obj.insert("⇉".to_string(), Value::Function {
            name: "⇉".to_string(),
            parameters: vec!["connection".to_string(), "address".to_string(), "port".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        net_obj.insert("⇓".to_string(), Value::Function {
            name: "⇓".to_string(),
            parameters: vec!["url".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        net_obj.insert("⇑".to_string(), Value::Function {
            name: "⇑".to_string(),
            parameters: vec!["url".to_string(), "data".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        net_obj.insert("⥮".to_string(), Value::Function {
            name: "⥮".to_string(),
            parameters: vec!["url".to_string(), "handler".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        // Define network object in environment
        self.environment.define("⚡".to_string(), Value::Object(net_obj.clone()));
    }

    fn init_concurrency_library(&mut self) {
        let mut conc_obj = HashMap::new();
        
        // Define concurrency methods
        conc_obj.insert("⟿".to_string(), Value::Function {
            name: "⟿".to_string(),
            parameters: vec!["size".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        conc_obj.insert("⇢".to_string(), Value::Function {
            name: "⇢".to_string(),
            parameters: vec!["channel".to_string(), "value".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        conc_obj.insert("⇠".to_string(), Value::Function {
            name: "⇠".to_string(),
            parameters: vec!["channel".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        conc_obj.insert("⟰".to_string(), Value::Function {
            name: "⟰".to_string(),
            parameters: vec!["name".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        conc_obj.insert("⇡".to_string(), Value::Function {
            name: "⇡".to_string(),
            parameters: vec!["state".to_string(), "key".to_string(), "value".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        conc_obj.insert("⇣".to_string(), Value::Function {
            name: "⇣".to_string(),
            parameters: vec!["state".to_string(), "key".to_string()],
            body: Box::new(ASTNode {
                node_type: NodeType::Block(vec![]),
                line: 0,
                column: 0,
            }),
            closure: Arc::new(Environment::new()),
        });

        // Define concurrency object in environment
        self.environment.define("⚯".to_string(), Value::Object(conc_obj.clone()));
    }

    pub fn run_ui(&mut self, args: Vec<Value>) -> Result<Value, LangError> {
        match args.get(0) {
            Some(Value::UI(ui)) => {
                // Start the UI and keep it running
                ui.start().map_err(|e| LangError::runtime_error(&format!("Failed to start UI: {}", e)))?;
                Ok(Value::Void)
            }
            _ => Err(LangError::runtime_error("Invalid arguments for UI creation"))
        }
    }

    fn call_method(&mut self, obj: Value, method: &str, args: Vec<Value>) -> Result<Value, LangError> {
        debug!("Calling method {} with {} arguments", method, args.len());
        match obj {
            Value::UI(ui) => {
                match method {
                    "start" => {
                        if args.len() != 0 {
                            return Err(LangError::runtime_error("start() takes no arguments"));
                        }
                        // This will block until the window is closed
                        ui.start()
                            .map_err(|e| LangError::runtime_error(&format!("Failed to start UI: {}", e)))
                            .map(|_| Value::Void)
                    },
                    "create_window" | "□" => {
                        if args.len() != 3 {
                            return Err(LangError::runtime_error("create_window() takes 3 arguments: title, width, height"));
                        }
                        let title = args[0].as_string().ok_or_else(|| LangError::runtime_error("First argument must be a string"))?;
                        let width = args[1].as_number().ok_or_else(|| LangError::runtime_error("Second argument must be a number"))? as f64;
                        let height = args[2].as_number().ok_or_else(|| LangError::runtime_error("Third argument must be a number"))? as f64;
                        ui.create_window(title.to_string(), width, height);
                        Ok(Value::Void)
                    },
                    "add_text" | "⬚" => {
                        if args.len() != 1 {
                            return Err(LangError::runtime_error("add_text() takes 1 argument: text"));
                        }
                        let text = args[0].as_string().ok_or_else(|| LangError::runtime_error("First argument must be a string"))?;
                        ui.add_text(text.to_string());
                        Ok(Value::Void)
                    },
                    _ => Err(LangError::runtime_error(&format!("Unknown method: {}", method))),
                }
            },
            Value::Object(ref map) => {
                if let Some(method_value) = map.get(method) {
                    match method_value {
                        Value::Function { name, parameters, body, closure } => {
                            if args.len() != parameters.len() {
                                return Err(LangError::runtime_error(&format!(
                                    "{}() takes {} arguments, but {} were provided",
                                    name,
                                    parameters.len(),
                                    args.len()
                                )));
                            }

                            let mut new_env = Environment::with_parent(closure.clone());
                            new_env.define("self".to_string(), obj.clone());
                            
                            for (param, arg) in parameters.iter().zip(args) {
                                new_env.define(param.clone(), arg);
                            }

        let mut interpreter = Interpreter::new();
                            interpreter.environment = new_env;
                            interpreter.interpret_node(body)
                        },
                        _ => Err(LangError::runtime_error(&format!("{} is not a function", method))),
                    }
                } else {
                    Err(LangError::runtime_error(&format!("Unknown method: {}", method)))
                }
            },
            Value::Function { ref name, ref parameters, ref body, ref closure } => {
                if args.len() != parameters.len() {
                    return Err(LangError::runtime_error(&format!(
                        "{}() takes {} arguments, but {} were provided",
                        name,
                        parameters.len(),
                        args.len()
                    )));
                }

                let mut new_env = Environment::with_parent(closure.clone());
                new_env.define("self".to_string(), obj.clone());
                
                for (param, arg) in parameters.iter().zip(args) {
                    new_env.define(param.clone(), arg);
                }

        let mut interpreter = Interpreter::new();
                interpreter.environment = new_env;
                interpreter.interpret_node(&body)
            },
            _ => Err(LangError::runtime_error(&format!("Cannot call methods on {:?}", obj))),
        }
    }

    fn run_program(&mut self) -> Result<Value, LangError> {
        if let Err(e) = self.run_ui(vec![Value::UI(self.ui.clone())]) {
            return Err(LangError::runtime_error(&format!("Failed to run UI: {}", e)));
        }
        Ok(Value::Void)
    }
}
