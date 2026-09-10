use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use serde::Deserialize;

use super::key_path::KeyPath;

/// The result of normalizing Blend's sparse Nickel authoring surface for one
/// concrete Target value.
#[derive(Debug, Clone)]
pub struct ResolutionPlan {
    pub source: Option<serde_json::Value>,
    pub automatic: HashSet<KeyPath>,
    pub manual: HashSet<KeyPath>,
    pub resource: ResourceDisposition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ResourceDisposition {
    #[default]
    Present,
    Absent,
    Unmanaged,
    Unresolved,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ResolutionNode {
    Record {
        children: HashMap<String, ResolutionNode>,
    },
    Value {
        requirement: Requirement,
        value: serde_json::Value,
    },
    Absent {
        requirement: Requirement,
    },
    Unmanaged,
    Assert {
        satisfied: bool,
    },
}

impl ResolutionNode {
    /// Materialize a missing-target evaluation without enforcing assertions.
    /// The resulting Source value can be evaluated as the next Target during
    /// `blend check`, while nodes omitted at runtime remain omitted here too.
    pub(crate) fn into_validation_probe(self) -> Option<serde_json::Value> {
        match self {
            Self::Record { children } => Some(serde_json::Value::Object(
                children
                    .into_iter()
                    .filter_map(|(key, child)| {
                        child.into_validation_probe().map(|value| (key, value))
                    })
                    .collect(),
            )),
            Self::Value { value, .. } => Some(value),
            Self::Absent { .. } | Self::Unmanaged | Self::Assert { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Requirement {
    Unresolved,
    Enforce,
}

impl ResolutionPlan {
    pub(crate) fn from_node(
        root: ResolutionNode,
        target: Option<&serde_json::Value>,
    ) -> Result<Self> {
        let root_unmanaged = matches!(&root, ResolutionNode::Unmanaged);
        let root_unresolved = matches!(
            &root,
            ResolutionNode::Absent {
                requirement: Requirement::Unresolved
            }
        );
        let mut automatic = HashSet::new();
        let mut manual = HashSet::new();
        let source = materialize(root, target, &KeyPath::root(), &mut automatic, &mut manual)?;
        let resource = if root_unmanaged {
            ResourceDisposition::Unmanaged
        } else if root_unresolved {
            ResourceDisposition::Unresolved
        } else if source.is_none() {
            ResourceDisposition::Absent
        } else {
            ResourceDisposition::Present
        };
        Ok(Self {
            source,
            automatic,
            manual,
            resource,
        })
    }
}

fn materialize(
    node: ResolutionNode,
    target: Option<&serde_json::Value>,
    path: &KeyPath,
    automatic: &mut HashSet<KeyPath>,
    manual: &mut HashSet<KeyPath>,
) -> Result<Option<serde_json::Value>> {
    match node {
        ResolutionNode::Record { children } => {
            let target_object = target.and_then(serde_json::Value::as_object);
            let mut object = serde_json::Map::new();
            for (key, child) in children {
                let child_path = path.child(key.clone());
                let child_target = target_object.and_then(|object| object.get(&key));
                if let Some(value) =
                    materialize(child, child_target, &child_path, automatic, manual)?
                {
                    object.insert(key, value);
                }
            }
            Ok(Some(serde_json::Value::Object(object)))
        }
        ResolutionNode::Value { requirement, value } => {
            if requirement == Requirement::Enforce {
                automatic.insert(path.clone());
            }
            Ok(Some(value))
        }
        ResolutionNode::Absent { requirement } => {
            if requirement == Requirement::Enforce {
                automatic.insert(path.clone());
            } else {
                manual.insert(path.clone());
            }
            Ok(None)
        }
        ResolutionNode::Unmanaged => Ok(target.cloned()),
        ResolutionNode::Assert { satisfied } => {
            if !satisfied {
                bail!("assertion failed at {}", display_path(path));
            }
            Ok(target.cloned())
        }
    }
}

fn display_path(path: &KeyPath) -> String {
    if path.is_root() {
        "<resource root>".to_string()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_enforcement_absence_and_unmanaged_nodes() {
        let node: ResolutionNode = serde_json::from_value(serde_json::json!({
            "kind": "record",
            "children": {
                "keep": {"kind": "unmanaged"},
                "remove": {"kind": "absent", "requirement": "enforce"},
                "set": {"kind": "value", "requirement": "enforce", "value": 2}
            }
        }))
        .unwrap();
        let target = serde_json::json!({"keep": 1, "remove": true, "set": 1});
        let plan = ResolutionPlan::from_node(node, Some(&target)).unwrap();

        assert_eq!(plan.source, Some(serde_json::json!({"keep": 1, "set": 2})));
        assert_eq!(plan.automatic.len(), 2);
        assert!(plan.manual.is_empty());
        assert_eq!(plan.resource, ResourceDisposition::Present);
    }

    #[test]
    fn failed_assertion_is_an_error() {
        let node: ResolutionNode = serde_json::from_value(serde_json::json!({
            "kind": "record",
            "children": {"token": {"kind": "assert", "satisfied": false}}
        }))
        .unwrap();
        let error =
            ResolutionPlan::from_node(node, Some(&serde_json::json!({"token": "x"}))).unwrap_err();
        assert!(error.to_string().contains("token"));
    }

    #[test]
    fn preserves_root_absence_unmanaged_and_unresolved() {
        for (node, resource, automatic, manual) in [
            (
                serde_json::json!({"kind": "absent", "requirement": "enforce"}),
                ResourceDisposition::Absent,
                true,
                false,
            ),
            (
                serde_json::json!({"kind": "unmanaged"}),
                ResourceDisposition::Unmanaged,
                false,
                false,
            ),
            (
                serde_json::json!({"kind": "absent", "requirement": "unresolved"}),
                ResourceDisposition::Unresolved,
                false,
                true,
            ),
        ] {
            let plan =
                ResolutionPlan::from_node(serde_json::from_value(node).unwrap(), None).unwrap();
            assert_eq!(plan.source, None);
            assert_eq!(plan.resource, resource);
            assert_eq!(plan.automatic.contains(&KeyPath::root()), automatic);
            assert_eq!(plan.manual.contains(&KeyPath::root()), manual);
        }
    }

    #[test]
    fn validation_probe_matches_missing_target_materialization() {
        let node: ResolutionNode = serde_json::from_value(serde_json::json!({
            "kind": "record",
            "children": {
                "asserted": {"kind": "assert", "satisfied": false},
                "kept": {"kind": "value", "requirement": "unresolved", "value": 1},
                "removed": {"kind": "absent", "requirement": "enforce"},
                "unmanaged": {"kind": "unmanaged"}
            }
        }))
        .unwrap();

        assert_eq!(
            node.into_validation_probe(),
            Some(serde_json::json!({"kept": 1}))
        );
    }
}
