// Tutorial Agent module for Anarchy Inference
//
// This module provides an interactive, step-by-step tutorial system
// for learning Anarchy Inference concepts and features.

use super::{
    OnboardingContext, 
    Tutorial, 
    TutorialStep,
    Exercise,
    ValidationResult,
    ValidationIssue,
    SkillLevel
};
use crate::parser::Parser;
use crate::lexer::Lexer;
use crate::language_hub_server::repl::session::Session;
use std::collections::HashMap;

/// Agent for providing interactive tutorials
pub struct TutorialAgent {
    /// Tutorial execution engine
    tutorial_engine: TutorialEngine,
    
    /// Exercise validator
    exercise_validator: ExerciseValidator,
    
    /// Progress tracker
    progress_tracker: ProgressTracker,
}

/// Engine for executing tutorials
struct TutorialEngine {
    /// Current tutorial
    current_tutorial: Option<String>,
    
    /// Current step
    current_step: usize,
    
    /// Tutorial execution state
    execution_state: TutorialExecutionState,
}

/// State of tutorial execution
enum TutorialExecutionState {
    /// Not started
    NotStarted,
    
    /// Reading instructions
    ReadingInstructions,
    
    /// Working on exercise
    WorkingOnExercise,
    
    /// Completed step
    CompletedStep,
    
    /// Completed tutorial
    CompletedTutorial,
}

/// Validator for exercises
struct ExerciseValidator {
    /// Custom validators by exercise ID
    custom_validators: HashMap<String, fn(&str) -> ValidationResult>,
}

/// Tracker for user progress
struct ProgressTracker {
    /// Completed tutorials
    completed_tutorials: Vec<String>,
    
    /// Completed steps by tutorial
    completed_steps: HashMap<String, Vec<usize>>,
    
    /// Exercise attempts by exercise ID
    exercise_attempts: HashMap<String, usize>,
    
    /// Skill levels by topic
    skill_levels: HashMap<String, SkillLevel>,
}

impl TutorialAgent {
    /// Create a new tutorial agent
    pub fn new() -> Self {
        TutorialAgent {
            tutorial_engine: TutorialEngine {
                current_tutorial: None,
                current_step: 0,
                execution_state: TutorialExecutionState::NotStarted,
            },
            exercise_validator: ExerciseValidator {
                custom_validators: HashMap::new(),
            },
            progress_tracker: ProgressTracker {
                completed_tutorials: Vec::new(),
                completed_steps: HashMap::new(),
                exercise_attempts: HashMap::new(),
                skill_levels: HashMap::new(),
            },
        }
    }
    
    /// Start a tutorial
    pub fn start_tutorial(&mut self, context: &mut OnboardingContext, tutorial_id: &str) -> Result<&Tutorial, String> {
        // Get the tutorial from the knowledge base
        let tutorial = match context.knowledge_base.tutorials.get(tutorial_id) {
            Some(tutorial) => tutorial,
            None => return Err(format!("Tutorial '{}' not found", tutorial_id)),
        };
        
        // Check prerequisites
        for prerequisite in &tutorial.prerequisites {
            if !context.progress.completed_tutorials.contains(prerequisite) {
                return Err(format!("Prerequisite tutorial '{}' not completed", prerequisite));
            }
        }
        
        // Update context
        context.progress.current_tutorial = Some(tutorial_id.to_string());
        context.progress.current_step = 0;
        
        // Update engine state
        self.tutorial_engine.current_tutorial = Some(tutorial_id.to_string());
        self.tutorial_engine.current_step = 0;
        self.tutorial_engine.execution_state = TutorialExecutionState::ReadingInstructions;
        
        Ok(tutorial)
    }
    
    /// Get the current tutorial step
    pub fn get_current_step(&self, context: &OnboardingContext) -> Option<&TutorialStep> {
        let tutorial_id = match &context.progress.current_tutorial {
            Some(id) => id,
            None => return None,
        };
        
        let tutorial = match context.knowledge_base.tutorials.get(tutorial_id) {
            Some(tutorial) => tutorial,
            None => return None,
        };
        
        tutorial.steps.get(context.progress.current_step)
    }
    
    /// Move to the next step in the tutorial
    pub fn next_step(&mut self, context: &mut OnboardingContext) -> Result<Option<&TutorialStep>, String> {
        let tutorial_id = match &context.progress.current_tutorial {
            Some(id) => id.clone(),
            None => return Err("No tutorial in progress".to_string()),
        };
        
        let tutorial = match context.knowledge_base.tutorials.get(&tutorial_id) {
            Some(tutorial) => tutorial,
            None => return Err(format!("Tutorial '{}' not found", tutorial_id)),
        };
        
        // Check if we're at the last step
        if context.progress.current_step >= tutorial.steps.len() - 1 {
            // Mark tutorial as completed
            if !context.progress.completed_tutorials.contains(&tutorial_id) {
                context.progress.completed_tutorials.push(tutorial_id.clone());
            }
            
            // Reset current tutorial
            context.progress.current_tutorial = None;
            context.progress.current_step = 0;
            
            // Update engine state
            self.tutorial_engine.current_tutorial = None;
            self.tutorial_engine.current_step = 0;
            self.tutorial_engine.execution_state = TutorialExecutionState::CompletedTutorial;
            
            return Ok(None);
        }
        
        // Move to the next step
        context.progress.current_step += 1;
        self.tutorial_engine.current_step += 1;
        self.tutorial_engine.execution_state = TutorialExecutionState::ReadingInstructions;
        
        // Get the new step
        Ok(tutorial.steps.get(context.progress.current_step))
    }
    
    /// Submit a solution for the current exercise
    pub fn submit_exercise(&mut self, context: &mut OnboardingContext, code: &str) -> Result<ValidationResult, String> {
        let tutorial_id = match &context.progress.current_tutorial {
            Some(id) => id.clone(),
            None => return Err("No tutorial in progress".to_string()),
        };
        
        let tutorial = match context.knowledge_base.tutorials.get(&tutorial_id) {
            Some(tutorial) => tutorial,
            None => return Err(format!("Tutorial '{}' not found", tutorial_id)),
        };
        
        let step = match tutorial.steps.get(context.progress.current_step) {
            Some(step) => step,
            None => return Err("Invalid tutorial step".to_string()),
        };
        
        let exercise = match &step.exercise {
            Some(exercise) => exercise,
            None => return Err("Current step does not have an exercise".to_string()),
        };
        
        // Track attempt
        let exercise_id = format!("{}_{}", tutorial_id, context.progress.current_step);
        let attempts = self.progress_tracker.exercise_attempts.entry(exercise_id.clone()).or_insert(0);
        *attempts += 1;
        
        // Validate the solution
        let result = (exercise.validation_fn)(code);
        
        // If correct, mark step as completed
        if result.is_correct {
            let completed_steps = self.progress_tracker.completed_steps.entry(tutorial_id.clone()).or_insert_with(Vec::new);
            if !completed_steps.contains(&context.progress.current_step) {
                completed_steps.push(context.progress.current_step);
            }
            
            self.tutorial_engine.execution_state = TutorialExecutionState::CompletedStep;
        }
        
        Ok(result)
    }
    
    /// Get a hint for the current step
    pub fn get_hint(&self, context: &OnboardingContext) -> Option<String> {
        let step = self.get_current_step(context)?;
        
        // Get the appropriate hint based on attempts
        let exercise_id = format!("{}_{}", context.progress.current_tutorial.as_ref()?, context.progress.current_step);
        let attempts = self.progress_tracker.exercise_attempts.get(&exercise_id).unwrap_or(&0);
        
        // Return a hint based on the number of attempts
        step.hints.get(*attempts % step.hints.len()).cloned()
    }
    
    /// Reset the current tutorial
    pub fn reset_tutorial(&mut self, context: &mut OnboardingContext) -> Result<(), String> {
        let tutorial_id = match &context.progress.current_tutorial {
            Some(id) => id.clone(),
            None => return Err("No tutorial in progress".to_string()),
        };
        
        // Reset progress for this tutorial
        self.progress_tracker.completed_steps.remove(&tutorial_id);
        
        // Reset exercise attempts for this tutorial
        let prefix = format!("{}_", tutorial_id);
        self.progress_tracker.exercise_attempts.retain(|k, _| !k.starts_with(&prefix));
        
        // Reset current step
        context.progress.current_step = 0;
        self.tutorial_engine.current_step = 0;
        self.tutorial_engine.execution_state = TutorialExecutionState::ReadingInstructions;
        
        Ok(())
    }
    
    /// Get recommended tutorials based on user progress
    pub fn get_recommended_tutorials(&self, context: &OnboardingContext) -> Vec<&Tutorial> {
        let mut recommended = Vec::new();
        
        for (id, tutorial) in &context.knowledge_base.tutorials {
            // Skip completed tutorials
            if context.progress.completed_tutorials.contains(id) {
                continue;
            }
            
            // Check if all prerequisites are met
            let prerequisites_met = tutorial.prerequisites.iter()
                .all(|prereq| context.progress.completed_tutorials.contains(prereq));
            
            if prerequisites_met {
                recommended.push(tutorial);
            }
        }
        
        // Sort by difficulty
        recommended.sort_by(|a, b| {
            use super::DifficultyLevel::*;
            let a_val = match a.difficulty {
                Beginner => 1,
                Intermediate => 2,
                Advanced => 3,
                Expert => 4,
            };
            
            let b_val = match b.difficulty {
                Beginner => 1,
                Intermediate => 2,
                Advanced => 3,
                Expert => 4,
            };
            
            a_val.cmp(&b_val)
        });
        
        recommended
    }
    
    /// Execute code in the tutorial context
    pub fn execute_code(&self, context: &OnboardingContext, code: &str) -> Result<String, String> {
        // Get the current session
        let session = match &context.current_session {
            Some(session) => session,
            None => return Err("No active REPL session".to_string()),
        };
        
        // Execute the code (simplified implementation)
        Ok("Code execution result would be here".to_string())
    }
    
    /// Register a custom validator for an exercise
    pub fn register_validator(&mut self, exercise_id: &str, validator: fn(&str) -> ValidationResult) {
        self.exercise_validator.custom_validators.insert(exercise_id.to_string(), validator);
    }
    
    /// Update user skill levels based on completed tutorials
    pub fn update_skill_levels(&mut self, context: &OnboardingContext) {
        for tutorial_id in &context.progress.completed_tutorials {
            if let Some(tutorial) = context.knowledge_base.tutorials.get(tutorial_id) {
                // Extract topics from tags
                for tag in &tutorial.tags {
                    let current_level = self.progress_tracker.skill_levels.entry(tag.clone())
                        .or_insert(SkillLevel::Beginner);
                    
                    // Upgrade skill level based on tutorial difficulty
                    *current_level = match (&tutorial.difficulty, current_level) {
                        (super::DifficultyLevel::Beginner, SkillLevel::Beginner) => SkillLevel::Intermediate,
                        (super::DifficultyLevel::Intermediate, SkillLevel::Beginner) => SkillLevel::Intermediate,
                        (super::DifficultyLevel::Intermediate, SkillLevel::Intermediate) => SkillLevel::Advanced,
                        (super::DifficultyLevel::Advanced, SkillLevel::Intermediate) => SkillLevel::Advanced,
                        (super::DifficultyLevel::Advanced, SkillLevel::Advanced) => SkillLevel::Expert,
                        (super::DifficultyLevel::Expert, _) => SkillLevel::Expert,
                        _ => current_level.clone(),
                    };
                }
            }
        }
    }
}
