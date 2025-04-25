// src/reasoning/operations.rs - High-level reasoning operations

use crate::error::LangError;
use crate::value::Value;
use crate::ast::ASTNode;
use super::engine::ReasoningEngine;
use super::strategies::ReasoningType;
use super::planning::{Plan, PlanStep, PlanStatus, StepStatus};

/// Reasoning operations for the Anarchy Inference language
pub struct ReasoningOperations {
    /// The reasoning engine
    engine: ReasoningEngine,
}

impl ReasoningOperations {
    /// Create a new reasoning operations instance
    pub fn new(engine: ReasoningEngine) -> Self {
        Self { engine }
    }
    
    /// Execute a reasoning operation with the specified strategy
    pub fn reason(&self, input: Value, strategy_type: ReasoningType) -> Result<Value, LangError> {
        self.engine.reason(input, strategy_type)
    }
    
    /// Execute conditional reasoning
    pub fn reason_conditional(&self, condition: Value, true_case: Value, false_case: Value) -> Result<Value, LangError> {
        // Create the input for conditional reasoning
        let mut input = Value::empty_object();
        input.set_property("condition".to_string(), condition)?;
        input.set_property("true_case".to_string(), true_case)?;
        input.set_property("false_case".to_string(), false_case)?;
        
        // Execute the reasoning
        self.engine.reason(input, ReasoningType::Conditional)
    }
    
    /// Execute heuristic reasoning
    pub fn reason_heuristic(&self, goal: Value, options: Vec<Value>, utility_function: Option<Value>) -> Result<Value, LangError> {
        // Create the input for heuristic reasoning
        let mut input = Value::empty_object();
        input.set_property("goal".to_string(), goal)?;
        input.set_property("options".to_string(), Value::array(options))?;
        
        if let Some(utility_fn) = utility_function {
            input.set_property("utility_function".to_string(), utility_fn)?;
        }
        
        // Execute the reasoning
        self.engine.reason(input, ReasoningType::Heuristic)
    }
    
    /// Execute ReAct reasoning
    pub fn reason_react(&self, goal: Value, tools: Vec<String>, max_iterations: Option<usize>) -> Result<Value, LangError> {
        // Create the input for ReAct reasoning
        let mut input = Value::empty_object();
        input.set_property("goal".to_string(), goal)?;
        
        // Convert tools to array of strings
        let tools_array = tools.iter()
            .map(|tool| Value::string(tool))
            .collect();
        input.set_property("tools".to_string(), Value::array(tools_array))?;
        
        if let Some(iterations) = max_iterations {
            input.set_property("max_iterations".to_string(), Value::number(iterations as f64))?;
        }
        
        // Execute the reasoning
        self.engine.reason(input, ReasoningType::ReAct)
    }
    
    /// Execute self-reflection reasoning
    pub fn reason_reflect(&self, reasoning_trace: Value) -> Result<Value, LangError> {
        // Execute the reasoning
        self.engine.reason(reasoning_trace, ReasoningType::SelfReflection)
    }
    
    /// Execute multi-agent reasoning
    pub fn reason_multi_agent(&self, goal: Value, agents: Vec<Value>, coordination_strategy: String) -> Result<Value, LangError> {
        // Create the input for multi-agent reasoning
        let mut input = Value::empty_object();
        input.set_property("goal".to_string(), goal)?;
        input.set_property("agents".to_string(), Value::array(agents))?;
        input.set_property("coordination_strategy".to_string(), Value::string(coordination_strategy))?;
        
        // Execute the reasoning
        self.engine.reason(input, ReasoningType::MultiAgent)
    }
    
    /// Create a plan for achieving a goal
    pub fn plan(&self, goal: Value) -> Result<Plan, LangError> {
        self.engine.plan(goal)
    }
    
    /// Execute a plan step by step
    pub fn execute_plan(&self, plan: &mut Plan) -> Result<Value, LangError> {
        // Update plan status to in progress
        plan.update_status(PlanStatus::InProgress)?;
        
        // Execute each step in the plan
        let mut results = Vec::new();
        
        for i in 0..plan.steps.len() {
            // Get the current step
            let step = &mut plan.steps[i];
            
            // Update step status to in progress
            step.update_status(StepStatus::InProgress);
            
            // Create the input for the step
            let mut step_input = Value::empty_object();
            step_input.set_property("description".to_string(), Value::string(&step.description))?;
            
            // Add tools to the input
            let tools_array = step.tools.iter()
                .map(|tool| Value::string(tool))
                .collect();
            step_input.set_property("tools".to_string(), Value::array(tools_array))?;
            
            // Execute the step using the appropriate reasoning strategy
            let result = match self.engine.reason(step_input.clone(), step.reasoning_type.clone()) {
                Ok(value) => {
                    // Step succeeded
                    step.update_status(StepStatus::Completed);
                    value
                },
                Err(err) => {
                    // Step failed
                    step.update_status(StepStatus::Failed);
                    
                    // Update plan status if needed
                    if plan.has_failed_steps() {
                        plan.update_status(PlanStatus::Failed)?;
                    }
                    
                    return Err(err);
                }
            };
            
            // Add the result to the results array
            results.push(result);
        }
        
        // Update plan status to completed if all steps are completed
        if plan.is_completed() {
            plan.update_status(PlanStatus::Completed)?;
        }
        
        // Create the final result
        let mut final_result = Value::empty_object();
        final_result.set_property("plan".to_string(), plan.to_value()?)?;
        
        // Convert results to array
        final_result.set_property("results".to_string(), Value::array(results))?;
        
        Ok(final_result)
    }
    
    /// Evaluate a result against a goal
    pub fn evaluate(&self, result: Value, goal: Value) -> Result<Value, LangError> {
        // Execute the evaluation
        let eval_result = self.engine.evaluate(result, goal)?;
        
        // Convert to Value
        let mut eval_obj = Value::empty_object();
        eval_obj.set_property("success".to_string(), Value::boolean(eval_result.success))?;
        eval_obj.set_property("score".to_string(), Value::number(eval_result.score))?;
        eval_obj.set_property("explanation".to_string(), Value::string(eval_result.explanation))?;
        
        Ok(eval_obj)
    }
    
    /// Reflect on a reasoning trace to improve it
    pub fn reflect(&self, reasoning_trace: Value) -> Result<Value, LangError> {
        // Execute the reflection
        let reflection_result = self.engine.reflect(reasoning_trace)?;
        
        // Convert to Value
        let mut reflection_obj = Value::empty_object();
        
        // Convert strengths to array
        let strengths_array = reflection_result.strengths.iter()
            .map(|s| Value::string(s))
            .collect();
        reflection_obj.set_property("strengths".to_string(), Value::array(strengths_array))?;
        
        // Convert weaknesses to array
        let weaknesses_array = reflection_result.weaknesses.iter()
            .map(|w| Value::string(w))
            .collect();
        reflection_obj.set_property("weaknesses".to_string(), Value::array(weaknesses_array))?;
        
        // Convert improvements to array
        let improvements_array = reflection_result.improvements.iter()
            .map(|i| Value::string(i))
            .collect();
        reflection_obj.set_property("improvements".to_string(), Value::array(improvements_array))?;
        
        // Add refined trace
        reflection_obj.set_property("refined_trace".to_string(), reflection_result.refined_trace)?;
        
        Ok(reflection_obj)
    }
    
    /// Get the reasoning engine
    pub fn get_engine(&self) -> &ReasoningEngine {
        &self.engine
    }
    
    /// Get a mutable reference to the reasoning engine
    pub fn get_engine_mut(&mut self) -> &mut ReasoningEngine {
        &mut self.engine
    }
}
