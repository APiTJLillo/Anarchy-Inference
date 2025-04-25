// src/reasoning/planning.rs - Planning system implementation

use crate::error::LangError;
use crate::value::Value;
use super::strategies::ReasoningType;

/// Status of a plan
#[derive(Debug, Clone, PartialEq)]
pub enum PlanStatus {
    /// Plan has not been started
    NotStarted,
    /// Plan is in progress
    InProgress,
    /// Plan has been completed successfully
    Completed,
    /// Plan has failed
    Failed,
}

/// Status of a plan step
#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently being executed
    InProgress,
    /// Step has been completed successfully
    Completed,
    /// Step has failed
    Failed,
}

/// A step in a plan
pub struct PlanStep {
    /// Description of the step
    pub description: String,
    /// Type of reasoning to use for this step
    pub reasoning_type: ReasoningType,
    /// Tools required for this step
    pub tools: Vec<String>,
    /// Status of the step
    pub status: StepStatus,
}

impl PlanStep {
    /// Create a new plan step
    pub fn new(description: String, reasoning_type: ReasoningType, tools: Vec<String>) -> Self {
        Self {
            description,
            reasoning_type,
            tools,
            status: StepStatus::Pending,
        }
    }
    
    /// Update the status of this step
    pub fn update_status(&mut self, status: StepStatus) {
        self.status = status;
    }
    
    /// Check if this step is completed
    pub fn is_completed(&self) -> bool {
        self.status == StepStatus::Completed
    }
    
    /// Check if this step has failed
    pub fn is_failed(&self) -> bool {
        self.status == StepStatus::Failed
    }
    
    /// Convert this step to a Value
    pub fn to_value(&self) -> Result<Value, LangError> {
        let mut step_obj = Value::empty_object();
        
        step_obj.set_property("description".to_string(), Value::string(&self.description))?;
        
        // Convert reasoning type to string
        let reasoning_type_str = match self.reasoning_type {
            ReasoningType::Conditional => "conditional",
            ReasoningType::Heuristic => "heuristic",
            ReasoningType::ReAct => "react",
            ReasoningType::SelfReflection => "self_reflection",
            ReasoningType::MultiAgent => "multi_agent",
        };
        step_obj.set_property("reasoning_type".to_string(), Value::string(reasoning_type_str))?;
        
        // Convert tools to array
        let tools_array = self.tools.iter()
            .map(|tool| Value::string(tool))
            .collect();
        step_obj.set_property("tools".to_string(), Value::array(tools_array))?;
        
        // Convert status to string
        let status_str = match self.status {
            StepStatus::Pending => "pending",
            StepStatus::InProgress => "in_progress",
            StepStatus::Completed => "completed",
            StepStatus::Failed => "failed",
        };
        step_obj.set_property("status".to_string(), Value::string(status_str))?;
        
        Ok(step_obj)
    }
}

/// A plan for achieving a goal
pub struct Plan {
    /// The goal of this plan
    pub goal: Value,
    /// Steps to achieve the goal
    pub steps: Vec<PlanStep>,
    /// Status of the plan
    pub status: PlanStatus,
}

impl Plan {
    /// Create a new plan
    pub fn new(goal: Value) -> Self {
        Self {
            goal,
            steps: Vec::new(),
            status: PlanStatus::NotStarted,
        }
    }
    
    /// Add a step to the plan
    pub fn add_step(&mut self, step: PlanStep) -> Result<(), LangError> {
        self.steps.push(step);
        Ok(())
    }
    
    /// Add a step to the plan from a Value
    pub fn add_step_from_value(&mut self, step_value: Value) -> Result<(), LangError> {
        if let Value::Complex(complex) = &step_value {
            let complex_ref = complex.borrow();
            if let Some(obj) = &complex_ref.object_data {
                // Extract the description
                let description = obj.get("description")
                    .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                    .ok_or_else(|| LangError::runtime_error("Step must include a 'description' field"))?;
                
                // Extract the reasoning type
                let reasoning_type_str = obj.get("reasoning_type")
                    .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("conditional");
                
                let reasoning_type = match reasoning_type_str {
                    "conditional" => ReasoningType::Conditional,
                    "heuristic" => ReasoningType::Heuristic,
                    "react" => ReasoningType::ReAct,
                    "self_reflection" => ReasoningType::SelfReflection,
                    "multi_agent" => ReasoningType::MultiAgent,
                    _ => ReasoningType::Conditional,
                };
                
                // Extract the tools
                let tools = obj.get("tools")
                    .and_then(|v| if let Value::Complex(c) = v {
                        let c_ref = c.borrow();
                        if let Some(arr) = &c_ref.array_data {
                            Some(arr.iter()
                                .filter_map(|item| if let Value::String(s) = item { Some(s.clone()) } else { None })
                                .collect())
                        } else { None }
                    } else { None })
                    .unwrap_or_else(Vec::new);
                
                // Create and add the step
                let step = PlanStep::new(description, reasoning_type, tools);
                self.add_step(step)?;
                
                return Ok(());
            }
        }
        
        Err(LangError::runtime_error("Invalid step value"))
    }
    
    /// Update the status of the plan
    pub fn update_status(&mut self, status: PlanStatus) -> Result<(), LangError> {
        self.status = status;
        Ok(())
    }
    
    /// Get the next step to execute
    pub fn get_next_step(&self) -> Option<&PlanStep> {
        self.steps.iter().find(|step| step.status == StepStatus::Pending)
    }
    
    /// Get a mutable reference to the next step to execute
    pub fn get_next_step_mut(&mut self) -> Option<&mut PlanStep> {
        self.steps.iter_mut().find(|step| step.status == StepStatus::Pending)
    }
    
    /// Check if all steps are completed
    pub fn is_completed(&self) -> bool {
        !self.steps.is_empty() && self.steps.iter().all(|step| step.is_completed())
    }
    
    /// Check if any step has failed
    pub fn has_failed_steps(&self) -> bool {
        self.steps.iter().any(|step| step.is_failed())
    }
    
    /// Convert this plan to a Value
    pub fn to_value(&self) -> Result<Value, LangError> {
        let mut plan_obj = Value::empty_object();
        
        plan_obj.set_property("goal".to_string(), self.goal.clone())?;
        
        // Convert steps to array
        let steps_array = self.steps.iter()
            .map(|step| step.to_value())
            .collect::<Result<Vec<Value>, LangError>>()?;
        plan_obj.set_property("steps".to_string(), Value::array(steps_array))?;
        
        // Convert status to string
        let status_str = match self.status {
            PlanStatus::NotStarted => "not_started",
            PlanStatus::InProgress => "in_progress",
            PlanStatus::Completed => "completed",
            PlanStatus::Failed => "failed",
        };
        plan_obj.set_property("status".to_string(), Value::string(status_str))?;
        
        Ok(plan_obj)
    }
}
