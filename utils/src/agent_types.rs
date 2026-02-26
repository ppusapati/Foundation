//! M5: Agent-Domain Compound Types – types specific to agent operations.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::id::{AgentDomain, DeterministicId, TaskDomain};
use foundation::primitives::{Percentage, SafeFloat, SafeInteger};
use foundation::time::LogicalTimestamp;
use foundation::types::PropertyMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Agent execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Planning,
    Executing,
    Waiting,
    Completed,
    Failed,
    Suspended,
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentState::Idle => write!(f, "IDLE"),
            AgentState::Planning => write!(f, "PLANNING"),
            AgentState::Executing => write!(f, "EXECUTING"),
            AgentState::Waiting => write!(f, "WAITING"),
            AgentState::Completed => write!(f, "COMPLETED"),
            AgentState::Failed => write!(f, "FAILED"),
            AgentState::Suspended => write!(f, "SUSPENDED"),
        }
    }
}

/// An agent profile describing capabilities and configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: DeterministicId<AgentDomain>,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub properties: PropertyMap,
    pub state: AgentState,
    pub created_at: LogicalTimestamp,
    pub updated_at: LogicalTimestamp,
}

impl AgentProfile {
    pub fn new(name: &str, role: &str, ts: LogicalTimestamp) -> Self {
        let id = DeterministicId::from_content(&format!("{}:{}", name, role));
        AgentProfile {
            id,
            name: name.to_string(),
            role: role.to_string(),
            capabilities: Vec::new(),
            properties: PropertyMap::new(),
            state: AgentState::Idle,
            created_at: ts,
            updated_at: ts,
        }
    }

    pub fn add_capability(&mut self, capability: &str) {
        if !self.capabilities.contains(&capability.to_string()) {
            self.capabilities.push(capability.to_string());
        }
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    pub fn transition_to(&mut self, new_state: AgentState, ts: LogicalTimestamp) -> FoundationResult<()> {
        let valid = match (self.state, new_state) {
            (AgentState::Idle, AgentState::Planning) => true,
            (AgentState::Planning, AgentState::Executing) => true,
            (AgentState::Executing, AgentState::Waiting) => true,
            (AgentState::Executing, AgentState::Completed) => true,
            (AgentState::Executing, AgentState::Failed) => true,
            (AgentState::Waiting, AgentState::Executing) => true,
            (_, AgentState::Suspended) => true,
            (AgentState::Suspended, AgentState::Idle) => true,
            (AgentState::Failed, AgentState::Idle) => true,
            (AgentState::Completed, AgentState::Idle) => true,
            _ => false,
        };
        if !valid {
            return Err(FoundationError::ValidationFailed(format!(
                "invalid state transition: {} -> {}",
                self.state, new_state
            )));
        }
        self.state = new_state;
        self.updated_at = ts;
        Ok(())
    }
}

/// Task priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// A task assigned to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub id: DeterministicId<TaskDomain>,
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_to: Option<DeterministicId<AgentDomain>>,
    pub depends_on: Vec<DeterministicId<TaskDomain>>,
    pub progress: Percentage,
    pub properties: PropertyMap,
    pub created_at: LogicalTimestamp,
    pub updated_at: LogicalTimestamp,
}

impl TaskDescriptor {
    pub fn new(name: &str, description: &str, priority: TaskPriority, ts: LogicalTimestamp) -> Self {
        let id = DeterministicId::from_content(&format!("{}:{}", name, ts.value()));
        TaskDescriptor {
            id,
            name: name.to_string(),
            description: description.to_string(),
            priority,
            status: TaskStatus::Pending,
            assigned_to: None,
            depends_on: Vec::new(),
            progress: Percentage::new(0.0).unwrap(),
            properties: PropertyMap::new(),
            created_at: ts,
            updated_at: ts,
        }
    }

    pub fn assign_to(&mut self, agent_id: DeterministicId<AgentDomain>, ts: LogicalTimestamp) {
        self.assigned_to = Some(agent_id);
        self.updated_at = ts;
    }

    pub fn update_status(&mut self, status: TaskStatus, ts: LogicalTimestamp) {
        self.status = status;
        self.updated_at = ts;
    }

    pub fn update_progress(&mut self, progress: Percentage, ts: LogicalTimestamp) {
        self.progress = progress;
        self.updated_at = ts;
    }

    pub fn is_blocked(&self, completed_tasks: &[DeterministicId<TaskDomain>]) -> bool {
        self.depends_on
            .iter()
            .any(|dep| !completed_tasks.contains(dep))
    }
}

/// Agent performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: DeterministicId<AgentDomain>,
    pub tasks_completed: SafeInteger,
    pub tasks_failed: SafeInteger,
    pub avg_completion_time: SafeFloat,
    pub success_rate: Percentage,
}

impl AgentMetrics {
    pub fn new(agent_id: DeterministicId<AgentDomain>) -> Self {
        AgentMetrics {
            agent_id,
            tasks_completed: SafeInteger::ZERO,
            tasks_failed: SafeInteger::ZERO,
            avg_completion_time: SafeFloat::ZERO,
            success_rate: Percentage::new(100.0).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_profile_creation() {
        let agent = AgentProfile::new("agent-1", "planner", LogicalTimestamp::ZERO);
        assert_eq!(agent.name, "agent-1");
        assert_eq!(agent.state, AgentState::Idle);
    }

    #[test]
    fn agent_state_transitions() {
        let mut agent = AgentProfile::new("agent-1", "worker", LogicalTimestamp::ZERO);
        assert!(agent.transition_to(AgentState::Planning, LogicalTimestamp::new(1)).is_ok());
        assert!(agent.transition_to(AgentState::Executing, LogicalTimestamp::new(2)).is_ok());
        assert!(agent.transition_to(AgentState::Completed, LogicalTimestamp::new(3)).is_ok());
    }

    #[test]
    fn agent_invalid_transition() {
        let mut agent = AgentProfile::new("agent-1", "worker", LogicalTimestamp::ZERO);
        assert!(agent.transition_to(AgentState::Completed, LogicalTimestamp::new(1)).is_err());
    }

    #[test]
    fn task_descriptor_creation() {
        let task = TaskDescriptor::new("task-1", "do stuff", TaskPriority::High, LogicalTimestamp::ZERO);
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn task_blocked_check() {
        let task = {
            let mut t = TaskDescriptor::new("t2", "after t1", TaskPriority::Medium, LogicalTimestamp::ZERO);
            t.depends_on.push(DeterministicId::from_content("t1"));
            t
        };
        let completed: Vec<DeterministicId<TaskDomain>> = vec![];
        assert!(task.is_blocked(&completed));

        let completed = vec![DeterministicId::from_content("t1")];
        assert!(!task.is_blocked(&completed));
    }

    #[test]
    fn agent_capabilities() {
        let mut agent = AgentProfile::new("a1", "worker", LogicalTimestamp::ZERO);
        agent.add_capability("code_review");
        agent.add_capability("code_review"); // duplicate
        assert_eq!(agent.capabilities.len(), 1);
        assert!(agent.has_capability("code_review"));
    }
}
