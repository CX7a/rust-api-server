use uuid::Uuid;
use chrono::Utc;

use crate::models::collaboration::{
    DocumentOperation, OperationType, ConflictDetection, ConflictResolution,
};

/// Operational Transformation engine for conflict resolution
pub struct OTEngine;

impl OTEngine {
    /// Transform operation against concurrent operations (Client-side OT)
    pub fn transform(
        client_op: &DocumentOperation,
        server_ops: &[DocumentOperation],
    ) -> DocumentOperation {
        let mut transformed_op = client_op.clone();

        for server_op in server_ops {
            transformed_op = Self::transform_against(&transformed_op, server_op);
        }

        transformed_op
    }

    /// Transform operation against single concurrent operation
    fn transform_against(
        base_op: &DocumentOperation,
        other_op: &DocumentOperation,
    ) -> DocumentOperation {
        match (&base_op.operation, &other_op.operation) {
            // Insert vs Insert
            (
                OperationType::Insert {
                    position: base_pos,
                    content: base_content,
                },
                OperationType::Insert {
                    position: other_pos, ..
                },
            ) => {
                let new_pos = if other_pos < base_pos {
                    base_pos + base_content.len()
                } else if other_pos == base_pos {
                    // Tie-break by operation ID
                    if base_op.id < other_op.id {
                        *base_pos
                    } else {
                        base_pos + base_content.len()
                    }
                } else {
                    *base_pos
                };

                let mut result = base_op.clone();
                if let OperationType::Insert { position, .. } = &mut result.operation {
                    *position = new_pos;
                }
                result
            }

            // Insert vs Delete
            (
                OperationType::Insert {
                    position: base_pos, ..
                },
                OperationType::Delete {
                    position: del_pos,
                    length: del_len,
                },
            ) => {
                let new_pos = if del_pos < base_pos && del_pos + del_len > base_pos {
                    // Delete range overlaps with insert position
                    *del_pos
                } else if del_pos < base_pos {
                    // Delete before insert
                    base_pos.saturating_sub(del_len)
                } else {
                    // Delete after insert
                    *base_pos
                };

                let mut result = base_op.clone();
                if let OperationType::Insert { position, .. } = &mut result.operation {
                    *position = new_pos;
                }
                result
            }

            // Delete vs Insert
            (
                OperationType::Delete {
                    position: base_pos,
                    length: base_len,
                },
                OperationType::Insert {
                    position: ins_pos,
                    content: ins_content,
                },
            ) => {
                let new_pos = if ins_pos < base_pos {
                    // Insert before delete
                    base_pos + ins_content.len()
                } else if ins_pos >= base_pos && ins_pos < base_pos + base_len {
                    // Insert within delete range - trim delete
                    let new_len = base_len.saturating_sub(1);
                    let mut result = base_op.clone();
                    if let OperationType::Delete { length, .. } = &mut result.operation {
                        *length = new_len;
                    }
                    return result;
                } else {
                    // Insert after delete
                    *base_pos
                };

                let mut result = base_op.clone();
                if let OperationType::Delete { position, .. } = &mut result.operation {
                    *position = new_pos;
                }
                result
            }

            // Delete vs Delete
            (
                OperationType::Delete {
                    position: base_pos,
                    length: base_len,
                },
                OperationType::Delete {
                    position: other_pos,
                    length: other_len,
                },
            ) => {
                let (new_pos, new_len) = if other_pos < base_pos {
                    if other_pos + other_len > base_pos {
                        // Other delete overlaps with base delete
                        let overlap = (other_pos + other_len) - base_pos;
                        (
                            *other_pos,
                            base_len.saturating_sub(overlap.min(base_len as usize) as usize),
                        )
                    } else {
                        // Other delete fully before base delete
                        (base_pos.saturating_sub(other_len), *base_len)
                    }
                } else if other_pos >= base_pos && other_pos < base_pos + base_len {
                    // Other delete overlaps with base delete
                    let overlap_end = (base_pos + base_len).min(other_pos + other_len);
                    let new_delete_len = (overlap_end - base_pos).max(other_pos - base_pos);
                    (*base_pos, new_delete_len)
                } else {
                    // Other delete fully after base delete
                    (*base_pos, *base_len)
                };

                let mut result = base_op.clone();
                if let OperationType::Delete {
                    position,
                    length,
                } = &mut result.operation
                {
                    *position = new_pos;
                    *length = new_len;
                }
                result
            }

            // Replace against Insert/Delete
            (OperationType::Replace { .. }, _) | (_, OperationType::Replace { .. }) => {
                // Replace is treated as delete + insert
                base_op.clone()
            }
        }
    }

    /// Detect conflicts between operations
    pub fn detect_conflicts(
        client_version: u32,
        server_ops: &[DocumentOperation],
    ) -> Option<ConflictDetection> {
        let conflicts: Vec<_> = server_ops
            .iter()
            .filter(|op| op.version >= client_version)
            .cloned()
            .collect();

        if conflicts.is_empty() {
            None
        } else {
            Some(ConflictDetection {
                session_id: Uuid::new_v4(),
                conflicting_operations: conflicts,
                detected_at: Utc::now(),
            })
        }
    }

    /// Resolve conflicts using merge-friendly approach
    pub fn resolve_conflicts(
        original_content: &str,
        conflicting_ops: &[DocumentOperation],
    ) -> ConflictResolution {
        let mut resolved_content = original_content.to_string();
        let mut transformed_ops = conflicting_ops.to_vec();

        // Sort operations by version and timestamp
        transformed_ops.sort_by(|a, b| {
            if a.version != b.version {
                a.version.cmp(&b.version)
            } else {
                a.timestamp.cmp(&b.timestamp)
            }
        });

        // Apply operations in order
        for op in &transformed_ops {
            resolved_content = Self::apply_operation(&resolved_content, op);
        }

        ConflictResolution {
            version: conflicting_ops.iter().map(|op| op.version).max().unwrap_or(0) + 1,
            resolved_content,
            conflicting_operations: transformed_ops,
            resolution_strategy: "operational_transformation".to_string(),
        }
    }

    /// Apply single operation to content
    fn apply_operation(content: &str, op: &DocumentOperation) -> String {
        match &op.operation {
            OperationType::Insert { position, content: text } => {
                let pos = (*position).min(content.len());
                let mut result = String::new();
                result.push_str(&content[..pos]);
                result.push_str(text);
                result.push_str(&content[pos..]);
                result
            }

            OperationType::Delete { position, length } => {
                let start = (*position).min(content.len());
                let end = (start + length).min(content.len());
                let mut result = String::new();
                result.push_str(&content[..start]);
                result.push_str(&content[end..]);
                result
            }

            OperationType::Replace {
                position,
                old_content: _,
                new_content: text,
            } => {
                let pos = (*position).min(content.len());
                let mut result = String::new();
                result.push_str(&content[..pos]);
                result.push_str(text);
                result.push_str(&content[pos..]);
                result
            }
        }
    }

    /// Validate operation feasibility
    pub fn validate_operation(
        op: &DocumentOperation,
        content_length: usize,
    ) -> Result<(), String> {
        match &op.operation {
            OperationType::Insert { position, content } => {
                if *position > content_length {
                    return Err(format!(
                        "Insert position {} exceeds content length {}",
                        position, content_length
                    ));
                }
                if content.is_empty() {
                    return Err("Insert content cannot be empty".to_string());
                }
                Ok(())
            }

            OperationType::Delete { position, length } => {
                if *position >= content_length {
                    return Err(format!(
                        "Delete position {} exceeds content length {}",
                        position, content_length
                    ));
                }
                if *length == 0 {
                    return Err("Delete length must be greater than 0".to_string());
                }
                if position + length > content_length {
                    return Err("Delete range exceeds content length".to_string());
                }
                Ok(())
            }

            OperationType::Replace {
                position,
                old_content,
                new_content,
            } => {
                if *position > content_length {
                    return Err(format!(
                        "Replace position {} exceeds content length {}",
                        position, content_length
                    ));
                }
                if old_content.is_empty() && new_content.is_empty() {
                    return Err("Replace must have non-empty old or new content".to_string());
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_insert_op(pos: usize, content: &str) -> DocumentOperation {
        DocumentOperation {
            id: Uuid::new_v4().to_string(),
            version: 1,
            timestamp: Utc::now(),
            user_id: Uuid::new_v4(),
            operation: OperationType::Insert {
                position: pos,
                content: content.to_string(),
            },
        }
    }

    fn make_delete_op(pos: usize, len: usize) -> DocumentOperation {
        DocumentOperation {
            id: Uuid::new_v4().to_string(),
            version: 1,
            timestamp: Utc::now(),
            user_id: Uuid::new_v4(),
            operation: OperationType::Delete {
                position: pos,
                length: len,
            },
        }
    }

    #[test]
    fn test_insert_insert_transform() {
        let op1 = make_insert_op(5, "hello");
        let op2 = make_insert_op(3, "world");

        let result = OTEngine::transform_against(&op1, &op2);

        if let OperationType::Insert { position, .. } = result.operation {
            assert_eq!(position, 10); // 5 + "world".len()
        } else {
            panic!("Expected Insert operation");
        }
    }

    #[test]
    fn test_insert_delete_transform() {
        let op1 = make_insert_op(5, "test");
        let op2 = make_delete_op(2, 3);

        let result = OTEngine::transform_against(&op1, &op2);

        if let OperationType::Insert { position, .. } = result.operation {
            assert_eq!(position, 2); // 5 - 3
        } else {
            panic!("Expected Insert operation");
        }
    }

    #[test]
    fn test_operation_validation() {
        let valid_insert = make_insert_op(5, "test");
        assert!(OTEngine::validate_operation(&valid_insert, 20).is_ok());

        let invalid_insert = make_insert_op(25, "test");
        assert!(OTEngine::validate_operation(&invalid_insert, 20).is_err());

        let valid_delete = make_delete_op(5, 3);
        assert!(OTEngine::validate_operation(&valid_delete, 20).is_ok());

        let invalid_delete = make_delete_op(15, 10);
        assert!(OTEngine::validate_operation(&invalid_delete, 20).is_err());
    }

    #[test]
    fn test_apply_operations() {
        let content = "hello world";

        let insert = make_insert_op(5, "!");
        let result = OTEngine::apply_operation(content, &insert);
        assert_eq!(result, "hello! world");

        let delete = make_delete_op(5, 6);
        let result = OTEngine::apply_operation(content, &delete);
        assert_eq!(result, "hello");
    }
}
