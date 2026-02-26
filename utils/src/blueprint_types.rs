//! M6: Blueprint-Domain Compound Types – types for blueprint/template operations.

use foundation::errors::{FoundationError, FoundationResult};
use foundation::id::{BlueprintDomain, DeterministicId};
use foundation::time::LogicalTimestamp;
use foundation::types::{PropertyMap, VersionInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Blueprint status in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlueprintStatus {
    Draft,
    Active,
    Deprecated,
    Archived,
}

impl fmt::Display for BlueprintStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlueprintStatus::Draft => write!(f, "DRAFT"),
            BlueprintStatus::Active => write!(f, "ACTIVE"),
            BlueprintStatus::Deprecated => write!(f, "DEPRECATED"),
            BlueprintStatus::Archived => write!(f, "ARCHIVED"),
        }
    }
}

/// A parameter definition within a blueprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
}

/// Supported parameter types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    List,
    Map,
}

impl ParameterDef {
    pub fn new(name: &str, param_type: ParameterType, required: bool) -> Self {
        ParameterDef {
            name: name.to_string(),
            param_type,
            required,
            default_value: None,
            description: String::new(),
        }
    }

    pub fn with_default(mut self, default: &str) -> Self {
        self.default_value = Some(default.to_string());
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
}

/// A step within a blueprint's execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStep {
    pub name: String,
    pub operation: String,
    pub parameters: BTreeMap<String, String>,
    pub depends_on: Vec<String>,
    pub description: String,
}

impl BlueprintStep {
    pub fn new(name: &str, operation: &str) -> Self {
        BlueprintStep {
            name: name.to_string(),
            operation: operation.to_string(),
            parameters: BTreeMap::new(),
            depends_on: Vec::new(),
            description: String::new(),
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_dependency(mut self, dep: &str) -> Self {
        self.depends_on.push(dep.to_string());
        self
    }
}

/// A blueprint definition – a reusable template for operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDef {
    pub id: DeterministicId<BlueprintDomain>,
    pub name: String,
    pub description: String,
    pub version: VersionInfo,
    pub status: BlueprintStatus,
    pub parameters: Vec<ParameterDef>,
    pub steps: Vec<BlueprintStep>,
    pub properties: PropertyMap,
    pub created_at: LogicalTimestamp,
    pub updated_at: LogicalTimestamp,
}

impl BlueprintDef {
    pub fn new(name: &str, description: &str, ts: LogicalTimestamp) -> Self {
        let id = DeterministicId::from_content(name);
        BlueprintDef {
            id,
            name: name.to_string(),
            description: description.to_string(),
            version: VersionInfo::new(0, 1, 0),
            status: BlueprintStatus::Draft,
            parameters: Vec::new(),
            steps: Vec::new(),
            properties: PropertyMap::new(),
            created_at: ts,
            updated_at: ts,
        }
    }

    pub fn add_parameter(&mut self, param: ParameterDef) {
        self.parameters.push(param);
    }

    pub fn add_step(&mut self, step: BlueprintStep) {
        self.steps.push(step);
    }

    pub fn activate(&mut self, ts: LogicalTimestamp) -> FoundationResult<()> {
        if self.status != BlueprintStatus::Draft {
            return Err(FoundationError::ValidationFailed(
                "only draft blueprints can be activated".into(),
            ));
        }
        self.status = BlueprintStatus::Active;
        self.updated_at = ts;
        Ok(())
    }

    pub fn deprecate(&mut self, ts: LogicalTimestamp) -> FoundationResult<()> {
        if self.status != BlueprintStatus::Active {
            return Err(FoundationError::ValidationFailed(
                "only active blueprints can be deprecated".into(),
            ));
        }
        self.status = BlueprintStatus::Deprecated;
        self.updated_at = ts;
        Ok(())
    }

    /// Validate that all step dependencies reference existing steps.
    pub fn validate_steps(&self) -> FoundationResult<()> {
        let step_names: Vec<&str> = self.steps.iter().map(|s| s.name.as_str()).collect();
        for step in &self.steps {
            for dep in &step.depends_on {
                if !step_names.contains(&dep.as_str()) {
                    return Err(FoundationError::ValidationFailed(format!(
                        "step '{}' depends on unknown step '{}'",
                        step.name, dep
                    )));
                }
            }
        }
        Ok(())
    }

    /// Validate that all required parameters have no missing definitions.
    pub fn validate_parameters(&self) -> FoundationResult<()> {
        let names: Vec<&str> = self.parameters.iter().map(|p| p.name.as_str()).collect();
        let unique_count = {
            let mut sorted = names.clone();
            sorted.sort();
            sorted.dedup();
            sorted.len()
        };
        if unique_count != names.len() {
            return Err(FoundationError::ValidationFailed(
                "duplicate parameter names".into(),
            ));
        }
        Ok(())
    }
}

/// An instantiation of a blueprint with concrete parameter values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintInstance {
    pub blueprint_id: DeterministicId<BlueprintDomain>,
    pub instance_id: String,
    pub parameter_values: BTreeMap<String, String>,
    pub created_at: LogicalTimestamp,
}

impl BlueprintInstance {
    pub fn new(
        blueprint: &BlueprintDef,
        parameter_values: BTreeMap<String, String>,
        ts: LogicalTimestamp,
    ) -> FoundationResult<Self> {
        // Check required parameters
        for param in &blueprint.parameters {
            if param.required
                && !parameter_values.contains_key(&param.name)
                && param.default_value.is_none()
            {
                return Err(FoundationError::ValidationFailed(format!(
                    "required parameter '{}' not provided",
                    param.name
                )));
            }
        }
        let instance_id = format!(
            "{}-{}",
            blueprint.id.short(),
            ts.value()
        );
        Ok(BlueprintInstance {
            blueprint_id: blueprint.id,
            instance_id,
            parameter_values,
            created_at: ts,
        })
    }

    pub fn get_param(&self, name: &str) -> Option<&String> {
        self.parameter_values.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blueprint_creation() {
        let bp = BlueprintDef::new("test-bp", "A test blueprint", LogicalTimestamp::ZERO);
        assert_eq!(bp.name, "test-bp");
        assert_eq!(bp.status, BlueprintStatus::Draft);
    }

    #[test]
    fn blueprint_lifecycle() {
        let mut bp = BlueprintDef::new("bp1", "desc", LogicalTimestamp::ZERO);
        assert!(bp.activate(LogicalTimestamp::new(1)).is_ok());
        assert_eq!(bp.status, BlueprintStatus::Active);
        assert!(bp.deprecate(LogicalTimestamp::new(2)).is_ok());
        assert_eq!(bp.status, BlueprintStatus::Deprecated);
    }

    #[test]
    fn blueprint_invalid_activate() {
        let mut bp = BlueprintDef::new("bp1", "desc", LogicalTimestamp::ZERO);
        bp.activate(LogicalTimestamp::new(1)).unwrap();
        assert!(bp.activate(LogicalTimestamp::new(2)).is_err());
    }

    #[test]
    fn blueprint_step_validation() {
        let mut bp = BlueprintDef::new("bp", "d", LogicalTimestamp::ZERO);
        bp.add_step(BlueprintStep::new("s1", "op1"));
        bp.add_step(BlueprintStep::new("s2", "op2").with_dependency("s1"));
        assert!(bp.validate_steps().is_ok());

        bp.add_step(BlueprintStep::new("s3", "op3").with_dependency("nonexistent"));
        assert!(bp.validate_steps().is_err());
    }

    #[test]
    fn blueprint_instance() {
        let mut bp = BlueprintDef::new("bp", "d", LogicalTimestamp::ZERO);
        bp.add_parameter(ParameterDef::new("name", ParameterType::String, true));
        bp.add_parameter(
            ParameterDef::new("count", ParameterType::Integer, false).with_default("10"),
        );

        let mut params = BTreeMap::new();
        params.insert("name".into(), "test".into());
        let instance = BlueprintInstance::new(&bp, params, LogicalTimestamp::new(1)).unwrap();
        assert_eq!(instance.get_param("name"), Some(&"test".to_string()));
    }

    #[test]
    fn blueprint_instance_missing_required() {
        let mut bp = BlueprintDef::new("bp", "d", LogicalTimestamp::ZERO);
        bp.add_parameter(ParameterDef::new("name", ParameterType::String, true));
        let params = BTreeMap::new();
        assert!(BlueprintInstance::new(&bp, params, LogicalTimestamp::new(1)).is_err());
    }
}
