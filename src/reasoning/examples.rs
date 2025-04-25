// src/reasoning/examples.rs - Examples of reasoning operations

use crate::error::LangError;
use crate::value::Value;
use crate::agent_memory::{AgentMemoryManager, MemorySegment, MemoryPriority};
use crate::external_tools::manager::ExternalToolManager;
use super::engine::ReasoningEngine;
use super::strategies::{
    ReasoningStrategy, 
    ReasoningType,
    ConditionalReasoning,
    HeuristicReasoning,
    ReActReasoning,
    SelfReflectionReasoning,
    MultiAgentReasoning
};
use super::memory_integration::MemoryContext;
use super::tool_integration::ToolManager;
use super::operations::ReasoningOperations;

/// Run examples of reasoning operations
pub fn run_examples() -> Result<(), LangError> {
    println!("Running reasoning examples...");
    
    // Set up the reasoning engine
    let engine = setup_reasoning_engine()?;
    
    // Create reasoning operations
    let operations = ReasoningOperations::new(engine);
    
    // Run examples
    conditional_reasoning_example(&operations)?;
    heuristic_reasoning_example(&operations)?;
    react_reasoning_example(&operations)?;
    planning_example(&operations)?;
    reflection_example(&operations)?;
    
    println!("All examples completed successfully!");
    
    Ok(())
}

/// Set up a reasoning engine for examples
fn setup_reasoning_engine() -> Result<ReasoningEngine, LangError> {
    // Create memory manager
    let memory_manager = AgentMemoryManager::new();
    
    // Create memory context
    let memory_context = MemoryContext::new(memory_manager);
    
    // Create tool manager
    let mut tool_manager = ToolManager::new();
    
    // Register example tools
    // In a real implementation, these would be actual tools
    // For now, we'll just use placeholders
    
    // Create reasoning engine
    let mut engine = ReasoningEngine::new(memory_context, tool_manager);
    
    // Register reasoning strategies
    engine.register_strategy(Box::new(ConditionalReasoning::new()));
    engine.register_strategy(Box::new(HeuristicReasoning::new()));
    engine.register_strategy(Box::new(ReActReasoning::new()));
    engine.register_strategy(Box::new(SelfReflectionReasoning::new()));
    engine.register_strategy(Box::new(MultiAgentReasoning::new()));
    
    Ok(engine)
}

/// Example of conditional reasoning
fn conditional_reasoning_example(operations: &ReasoningOperations) -> Result<(), LangError> {
    println!("\nConditional Reasoning Example:");
    
    // Create a condition
    let condition = Value::boolean(true);
    
    // Create true and false cases
    let true_case = Value::string("The condition is true");
    let false_case = Value::string("The condition is false");
    
    // Execute conditional reasoning
    let result = operations.reason_conditional(condition, true_case, false_case)?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}

/// Example of heuristic reasoning
fn heuristic_reasoning_example(operations: &ReasoningOperations) -> Result<(), LangError> {
    println!("\nHeuristic Reasoning Example:");
    
    // Create a goal
    let goal = Value::string("Find the fastest route to the destination");
    
    // Create options
    let options = vec![
        Value::string("Route A: 30 minutes, highway"),
        Value::string("Route B: 25 minutes, toll road"),
        Value::string("Route C: 40 minutes, scenic route"),
    ];
    
    // Execute heuristic reasoning
    let result = operations.reason_heuristic(goal, options, None)?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}

/// Example of ReAct reasoning
fn react_reasoning_example(operations: &ReasoningOperations) -> Result<(), LangError> {
    println!("\nReAct Reasoning Example:");
    
    // Create a goal
    let goal = Value::string("Find the population of France");
    
    // Specify tools
    let tools = vec![
        "search".to_string(),
        "extract_data".to_string(),
    ];
    
    // Execute ReAct reasoning
    let result = operations.reason_react(goal, tools, Some(3))?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}

/// Example of planning
fn planning_example(operations: &ReasoningOperations) -> Result<(), LangError> {
    println!("\nPlanning Example:");
    
    // Create a goal
    let mut goal = Value::empty_object();
    goal.set_property("destination".to_string(), Value::string("Paris"))?;
    goal.set_property("duration".to_string(), Value::string("7 days"))?;
    goal.set_property("budget".to_string(), Value::number(2000.0))?;
    
    let interests = vec![
        Value::string("art"),
        Value::string("history"),
        Value::string("food"),
    ];
    goal.set_property("interests".to_string(), Value::array(interests))?;
    
    // Create a plan
    let mut plan = operations.plan(goal)?;
    
    println!("Plan: {:?}", plan.to_value()?);
    
    // Execute the plan
    let result = operations.execute_plan(&mut plan)?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}

/// Example of reflection
fn reflection_example(operations: &ReasoningOperations) -> Result<(), LangError> {
    println!("\nReflection Example:");
    
    // Create a reasoning trace
    let mut reasoning_trace = Value::empty_object();
    reasoning_trace.set_property("goal".to_string(), Value::string("Find the capital of France"))?;
    
    let steps = vec![
        Value::string("I need to search for information about France"),
        Value::string("I'll use the search tool to find the capital"),
        Value::string("The search results show that Paris is the capital of France"),
        Value::string("Therefore, the capital of France is Paris"),
    ];
    reasoning_trace.set_property("steps".to_string(), Value::array(steps))?;
    
    // Execute reflection
    let result = operations.reflect(reasoning_trace)?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
