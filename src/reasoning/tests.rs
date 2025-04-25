// src/reasoning/tests.rs - Tests for reasoning operations

#[cfg(test)]
mod tests {
    use crate::error::LangError;
    use crate::value::Value;
    use crate::agent_memory::{AgentMemoryManager, MemorySegment, MemoryPriority};
    use crate::external_tools::manager::ExternalToolManager;
    use crate::reasoning::engine::ReasoningEngine;
    use crate::reasoning::strategies::{
        ReasoningStrategy, 
        ReasoningType,
        ConditionalReasoning,
        HeuristicReasoning,
        ReActReasoning,
        SelfReflectionReasoning,
        MultiAgentReasoning
    };
    use crate::reasoning::memory_integration::MemoryContext;
    use crate::reasoning::tool_integration::ToolManager;
    use crate::reasoning::operations::ReasoningOperations;
    use crate::reasoning::planning::{Plan, PlanStep, PlanStatus, StepStatus};

    // Helper function to set up a reasoning engine for tests
    fn setup_test_engine() -> ReasoningEngine {
        // Create memory manager
        let memory_manager = AgentMemoryManager::new();
        
        // Create memory context
        let memory_context = MemoryContext::new(memory_manager);
        
        // Create tool manager
        let tool_manager = ToolManager::new();
        
        // Create reasoning engine
        let mut engine = ReasoningEngine::new(memory_context, tool_manager);
        
        // Register reasoning strategies
        engine.register_strategy(Box::new(ConditionalReasoning::new()));
        engine.register_strategy(Box::new(HeuristicReasoning::new()));
        engine.register_strategy(Box::new(ReActReasoning::new()));
        engine.register_strategy(Box::new(SelfReflectionReasoning::new()));
        engine.register_strategy(Box::new(MultiAgentReasoning::new()));
        
        engine
    }

    #[test]
    fn test_conditional_reasoning() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
        // Test with true condition
        let condition_true = Value::boolean(true);
        let true_case = Value::string("True case");
        let false_case = Value::string("False case");
        
        let result_true = operations.reason_conditional(condition_true, true_case.clone(), false_case.clone())?;
        assert_eq!(result_true, true_case);
        
        // Test with false condition
        let condition_false = Value::boolean(false);
        let result_false = operations.reason_conditional(condition_false, true_case.clone(), false_case.clone())?;
        assert_eq!(result_false, false_case);
        
        Ok(())
    }

    #[test]
    fn test_heuristic_reasoning() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
        // Create a goal
        let goal = Value::string("Find the fastest route");
        
        // Create options
        let options = vec![
            Value::string("Route A: 30 minutes"),
            Value::string("Route B: 25 minutes"),
            Value::string("Route C: 40 minutes"),
        ];
        
        // Execute heuristic reasoning
        let result = operations.reason_heuristic(goal, options.clone(), None)?;
        
        // In our simplified implementation, it should return the first option
        assert_eq!(result, options[0]);
        
        Ok(())
    }

    #[test]
    fn test_planning() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
        // Create a goal
        let mut goal = Value::empty_object();
        goal.set_property("destination".to_string(), Value::string("Paris"))?;
        goal.set_property("duration".to_string(), Value::string("7 days"))?;
        
        // Create a plan
        let plan = operations.plan(goal.clone())?;
        
        // Verify that the plan has the correct goal
        assert_eq!(plan.goal, goal);
        
        // Verify that the plan status is NotStarted
        assert_eq!(plan.status, PlanStatus::NotStarted);
        
        Ok(())
    }

    #[test]
    fn test_reflection() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
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
        let result = operations.reflect(reasoning_trace.clone())?;
        
        // Verify that the result is an object
        if let Value::Complex(complex) = &result {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                // Verify that the result has the expected fields
                assert!(obj.contains_key("strengths"));
                assert!(obj.contains_key("weaknesses"));
                assert!(obj.contains_key("improvements"));
                assert!(obj.contains_key("refined_trace"));
                
                // Verify that the refined trace is present
                if let Some(refined_trace) = obj.get("refined_trace") {
                    assert_eq!(refined_trace, &reasoning_trace);
                } else {
                    panic!("Refined trace not found in reflection result");
                }
            } else {
                panic!("Reflection result is not an object");
            }
        } else {
            panic!("Reflection result is not a complex value");
        }
        
        Ok(())
    }

    #[test]
    fn test_react_reasoning() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
        // Create a goal
        let goal = Value::string("Find the population of France");
        
        // Specify tools
        let tools = vec![
            "search".to_string(),
            "extract_data".to_string(),
        ];
        
        // Execute ReAct reasoning
        let result = operations.reason_react(goal.clone(), tools, Some(3))?;
        
        // Verify that the result is an object
        if let Value::Complex(complex) = &result {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                // Verify that the result has the expected fields
                assert!(obj.contains_key("goal"));
                assert!(obj.contains_key("trace"));
                
                // Verify that the goal matches
                if let Some(result_goal) = obj.get("goal") {
                    assert_eq!(result_goal, &goal);
                } else {
                    panic!("Goal not found in ReAct result");
                }
            } else {
                panic!("ReAct result is not an object");
            }
        } else {
            panic!("ReAct result is not a complex value");
        }
        
        Ok(())
    }

    #[test]
    fn test_multi_agent_reasoning() -> Result<(), LangError> {
        // Set up the reasoning engine
        let engine = setup_test_engine();
        
        // Create reasoning operations
        let operations = ReasoningOperations::new(engine);
        
        // Create a goal
        let goal = Value::string("Solve a complex problem");
        
        // Create agents
        let agents = vec![
            Value::string("Agent 1: Research specialist"),
            Value::string("Agent 2: Analysis specialist"),
            Value::string("Agent 3: Decision maker"),
        ];
        
        // Execute multi-agent reasoning with hierarchical coordination
        let result = operations.reason_multi_agent(goal.clone(), agents.clone(), "hierarchical".to_string())?;
        
        // Verify that the result is an object
        if let Value::Complex(complex) = &result {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                // Verify that the result has the expected fields
                assert!(obj.contains_key("goal"));
                assert!(obj.contains_key("coordination"));
                assert!(obj.contains_key("result"));
                
                // Verify that the goal matches
                if let Some(result_goal) = obj.get("goal") {
                    assert_eq!(result_goal, &goal);
                } else {
                    panic!("Goal not found in multi-agent result");
                }
                
                // Verify that the coordination strategy is hierarchical
                if let Some(Value::String(coordination)) = obj.get("coordination") {
                    assert_eq!(coordination, "hierarchical");
                } else {
                    panic!("Coordination strategy not found or not a string in multi-agent result");
                }
            } else {
                panic!("Multi-agent result is not an object");
            }
        } else {
            panic!("Multi-agent result is not a complex value");
        }
        
        Ok(())
    }
}
