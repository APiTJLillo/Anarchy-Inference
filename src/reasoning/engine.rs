// src/reasoning/engine.rs - Core reasoning engine implementation

use crate::error::LangError;
use crate::value::Value;
use super::strategies::{ReasoningStrategy, ReasoningType};
use super::planning::{Plan, PlanStatus};
use super::memory_integration::MemoryContext;
use super::tool_integration::ToolManager;

/// Result of an evaluation operation
pub struct EvaluationResult {
    /// Whether the result meets the goal
    pub success: bool,
    /// Score between 0.0 and 1.0 indicating how well the result meets the goal
    pub score: f64,
    /// Explanation of the evaluation
    pub explanation: String,
}

/// Result of a reflection operation
pub struct ReflectionResult {
    /// Identified strengths in the reasoning
    pub strengths: Vec<String>,
    /// Identified weaknesses in the reasoning
    pub weaknesses: Vec<String>,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Refined reasoning trace
    pub refined_trace: Value,
}

/// The central reasoning engine that orchestrates the reasoning process
pub struct ReasoningEngine {
    /// Memory context for accessing agent memory
    memory_context: MemoryContext,
    /// Available reasoning strategies
    reasoning_strategies: Vec<Box<dyn ReasoningStrategy>>,
    /// Tool manager for external tool integration
    tool_manager: ToolManager,
}

impl ReasoningEngine {
    /// Create a new reasoning engine
    pub fn new(memory_context: MemoryContext, tool_manager: ToolManager) -> Self {
        Self {
            memory_context,
            reasoning_strategies: Vec::new(),
            tool_manager,
        }
    }
    
    /// Register a reasoning strategy
    pub fn register_strategy(&mut self, strategy: Box<dyn ReasoningStrategy>) {
        self.reasoning_strategies.push(strategy);
    }
    
    /// Get a strategy by type
    pub fn get_strategy(&self, strategy_type: ReasoningType) -> Option<&Box<dyn ReasoningStrategy>> {
        self.reasoning_strategies.iter().find(|s| s.get_type() == strategy_type)
    }
    
    /// Apply reasoning to an input using a specified strategy
    pub fn reason(&self, input: Value, strategy_type: ReasoningType) -> Result<Value, LangError> {
        // Get the specified strategy
        let strategy = self.get_strategy(strategy_type)
            .ok_or_else(|| LangError::runtime_error(&format!("Reasoning strategy {:?} not found", strategy_type)))?;
        
        // Apply the strategy
        strategy.apply(&self.memory_context, &input)
    }
    
    /// Create a plan for achieving a goal
    pub fn plan(&self, goal: Value) -> Result<Plan, LangError> {
        // Create a new plan with the specified goal
        let mut plan = Plan::new(goal.clone());
        
        // Use the planning strategy to decompose the goal into steps
        let planning_strategy = self.get_strategy(ReasoningType::Heuristic)
            .ok_or_else(|| LangError::runtime_error("Planning strategy not found"))?;
        
        // Apply the planning strategy to generate steps
        let plan_steps = planning_strategy.apply(&self.memory_context, &goal)?;
        
        // Parse the steps and add them to the plan
        // This assumes the planning strategy returns a Value containing an array of step objects
        if let Value::Complex(complex) = plan_steps {
            let complex_ref = complex.borrow();
            if let Some(steps) = &complex_ref.array_data {
                for step in steps {
                    // Parse each step and add it to the plan
                    // This is a simplified implementation - in practice, you would parse the step object
                    // to extract the description, reasoning type, tools, etc.
                    plan.add_step_from_value(step.clone())?;
                }
            }
        }
        
        Ok(plan)
    }
    
    /// Evaluate a result against a goal
    pub fn evaluate(&self, result: Value, goal: Value) -> Result<EvaluationResult, LangError> {
        // Use the evaluation strategy to assess how well the result meets the goal
        let evaluation_strategy = self.get_strategy(ReasoningType::SelfReflection)
            .ok_or_else(|| LangError::runtime_error("Evaluation strategy not found"))?;
        
        // Create a combined input for evaluation
        let mut eval_input = Value::empty_object();
        eval_input.set_property("result".to_string(), result)?;
        eval_input.set_property("goal".to_string(), goal)?;
        
        // Apply the evaluation strategy
        let eval_result = evaluation_strategy.apply(&self.memory_context, &eval_input)?;
        
        // Parse the evaluation result
        // This assumes the evaluation strategy returns a Value containing success, score, and explanation
        if let Value::Complex(complex) = eval_result {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                let success = obj.get("success")
                    .and_then(|v| if let Value::Boolean(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                
                let score = obj.get("score")
                    .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                    .unwrap_or(0.0);
                
                let explanation = obj.get("explanation")
                    .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "No explanation provided".to_string());
                
                return Ok(EvaluationResult {
                    success,
                    score,
                    explanation,
                });
            }
        }
        
        // If parsing fails, return a default result
        Err(LangError::runtime_error("Failed to parse evaluation result"))
    }
    
    /// Reflect on a reasoning trace to improve it
    pub fn reflect(&self, reasoning_trace: Value) -> Result<ReflectionResult, LangError> {
        // Use the reflection strategy to analyze the reasoning trace
        let reflection_strategy = self.get_strategy(ReasoningType::SelfReflection)
            .ok_or_else(|| LangError::runtime_error("Reflection strategy not found"))?;
        
        // Apply the reflection strategy
        let reflection_result = reflection_strategy.apply(&self.memory_context, &reasoning_trace)?;
        
        // Parse the reflection result
        // This assumes the reflection strategy returns a Value containing strengths, weaknesses, improvements, and refined_trace
        if let Value::Complex(complex) = reflection_result {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                // Extract strengths
                let strengths = obj.get("strengths")
                    .and_then(|v| if let Value::Complex(c) = v {
                        let c_ref = c.borrow();
                        if let Some(arr) = &c_ref.array_data {
                            Some(arr.iter()
                                .filter_map(|item| if let Value::String(s) = item { Some(s.clone()) } else { None })
                                .collect())
                        } else { None }
                    } else { None })
                    .unwrap_or_else(Vec::new);
                
                // Extract weaknesses
                let weaknesses = obj.get("weaknesses")
                    .and_then(|v| if let Value::Complex(c) = v {
                        let c_ref = c.borrow();
                        if let Some(arr) = &c_ref.array_data {
                            Some(arr.iter()
                                .filter_map(|item| if let Value::String(s) = item { Some(s.clone()) } else { None })
                                .collect())
                        } else { None }
                    } else { None })
                    .unwrap_or_else(Vec::new);
                
                // Extract improvements
                let improvements = obj.get("improvements")
                    .and_then(|v| if let Value::Complex(c) = v {
                        let c_ref = c.borrow();
                        if let Some(arr) = &c_ref.array_data {
                            Some(arr.iter()
                                .filter_map(|item| if let Value::String(s) = item { Some(s.clone()) } else { None })
                                .collect())
                        } else { None }
                    } else { None })
                    .unwrap_or_else(Vec::new);
                
                // Extract refined trace
                let refined_trace = obj.get("refined_trace")
                    .cloned()
                    .unwrap_or(reasoning_trace.clone());
                
                return Ok(ReflectionResult {
                    strengths,
                    weaknesses,
                    improvements,
                    refined_trace,
                });
            }
        }
        
        // If parsing fails, return a default result
        Err(LangError::runtime_error("Failed to parse reflection result"))
    }
    
    /// Get the memory context
    pub fn get_memory_context(&self) -> &MemoryContext {
        &self.memory_context
    }
    
    /// Get a mutable reference to the memory context
    pub fn get_memory_context_mut(&mut self) -> &mut MemoryContext {
        &mut self.memory_context
    }
    
    /// Get the tool manager
    pub fn get_tool_manager(&self) -> &ToolManager {
        &self.tool_manager
    }
    
    /// Get a mutable reference to the tool manager
    pub fn get_tool_manager_mut(&mut self) -> &mut ToolManager {
        &mut self.tool_manager
    }
}
