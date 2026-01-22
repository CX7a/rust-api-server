use dashmap::DashMap;
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::models::collaboration::{
    DocumentOperation, OperationType, CursorUpdate, ConflictDetection,
};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

pub struct CollaborationManager {
    // Session ID -> Participants and operations
    active_sessions: DashMap<Uuid, SessionState>,
    // Broadcast channel for each session
    channels: DashMap<Uuid, broadcast::Sender<DocumentOperation>>,
}

#[derive(Clone)]
struct SessionState {
    session_id: Uuid,
    file_id: Uuid,
    participants: HashMap<Uuid, ParticipantState>,
    operations: Vec<DocumentOperation>,
    version: u32,
}

#[derive(Clone)]
struct ParticipantState {
    user_id: Uuid,
    cursor_position: Option<i32>,
    selection_start: Option<i32>,
    selection_end: Option<i32>,
}

impl CollaborationManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            active_sessions: DashMap::new(),
            channels: DashMap::new(),
        })
    }

    /// Create new collaboration session
    pub fn create_session(&self, session_id: Uuid, file_id: Uuid) -> Result<(), String> {
        if self.active_sessions.contains_key(&session_id) {
            return Err("Session already exists".to_string());
        }

        let state = SessionState {
            session_id,
            file_id,
            participants: HashMap::new(),
            operations: Vec::new(),
            version: 0,
        };

        self.active_sessions.insert(session_id, state);

        // Create broadcast channel for this session
        let (tx, _) = broadcast::channel(1000);
        self.channels.insert(session_id, tx);

        Ok(())
    }

    /// Join user to session
    pub fn join_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), String> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            session.participants.insert(
                user_id,
                ParticipantState {
                    user_id,
                    cursor_position: None,
                    selection_start: None,
                    selection_end: None,
                },
            );
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Remove user from session
    pub fn leave_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), String> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            session.participants.remove(&user_id);
            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Update cursor position for user
    pub fn update_cursor(
        &self,
        session_id: Uuid,
        cursor_update: CursorUpdate,
    ) -> Result<(), String> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            if let Some(participant) = session.participants.get_mut(&cursor_update.user_id) {
                participant.cursor_position = Some(cursor_update.cursor_position);
                participant.selection_start = cursor_update.selection_start;
                participant.selection_end = cursor_update.selection_end;
                Ok(())
            } else {
                Err("User not in session".to_string())
            }
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Apply operation to document
    pub fn apply_operation(
        &self,
        session_id: Uuid,
        mut operation: DocumentOperation,
    ) -> Result<u32, String> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            // Transform against concurrent operations
            let concurrent_ops: Vec<_> = session
                .operations
                .iter()
                .filter(|op| op.version >= operation.version && op.user_id != operation.user_id)
                .cloned()
                .collect();

            if !concurrent_ops.is_empty() {
                operation = Self::transform_operation(&operation, &concurrent_ops);
            }

            session.operations.push(operation);
            session.version += 1;
            Ok(session.version)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Detect conflicts in operations
    pub fn detect_conflicts(
        &self,
        session_id: Uuid,
        incoming_version: u32,
    ) -> Result<Vec<DocumentOperation>, String> {
        if let Some(session) = self.active_sessions.get(&session_id) {
            let conflicts: Vec<_> = session
                .operations
                .iter()
                .filter(|op| op.version >= incoming_version)
                .cloned()
                .collect();

            Ok(conflicts)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Get all participants in session
    pub fn get_participants(&self, session_id: Uuid) -> Result<Vec<(Uuid, CursorUpdate)>, String> {
        if let Some(session) = self.active_sessions.get(&session_id) {
            let participants = session
                .participants
                .iter()
                .map(|(user_id, state)| {
                    (
                        *user_id,
                        CursorUpdate {
                            user_id: *user_id,
                            session_id,
                            cursor_position: state.cursor_position.unwrap_or(0),
                            selection_start: state.selection_start,
                            selection_end: state.selection_end,
                        },
                    )
                })
                .collect();

            Ok(participants)
        } else {
            Err("Session not found".to_string())
        }
    }

    /// Get broadcast channel for session
    pub fn get_channel(
        &self,
        session_id: Uuid,
    ) -> Result<broadcast::Sender<DocumentOperation>, String> {
        self.channels
            .get(&session_id)
            .map(|ch| ch.clone())
            .ok_or_else(|| "Session channel not found".to_string())
    }

    /// Transform operation against concurrent operations (OT)
    pub fn transform_operation(
        base_op: &DocumentOperation,
        concurrent_ops: &[DocumentOperation],
    ) -> DocumentOperation {
        let mut transformed_op = base_op.clone();

        for concurrent_op in concurrent_ops {
            transformed_op = Self::transform_against_single(&transformed_op, concurrent_op);
        }

        transformed_op
    }

    /// Transform single operation against concurrent operation
    fn transform_against_single(
        base_op: &DocumentOperation,
        concurrent_op: &DocumentOperation,
    ) -> DocumentOperation {
        match (&base_op.operation, &concurrent_op.operation) {
            // Insert vs Insert
            (
                OperationType::Insert {
                    position: base_pos,
                    content: base_content,
                },
                OperationType::Insert {
                    position: conc_pos, ..
                },
            ) => {
                let new_position = if conc_pos < base_pos {
                    base_pos + base_content.len()
                } else {
                    *base_pos
                };

                let mut new_op = base_op.clone();
                if let OperationType::Insert { position, .. } = &mut new_op.operation {
                    *position = new_position;
                }
                new_op
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
                let new_position = if del_pos <= base_pos {
                    base_pos.saturating_sub(del_len.min(base_pos - del_pos))
                } else {
                    *base_pos
                };

                let mut new_op = base_op.clone();
                if let OperationType::Insert { position, .. } = &mut new_op.operation {
                    *position = new_position;
                }
                new_op
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
                let new_position = if ins_pos < base_pos {
                    base_pos + ins_content.len()
                } else {
                    *base_pos
                };

                let mut new_op = base_op.clone();
                if let OperationType::Delete { position, .. } = &mut new_op.operation {
                    *position = new_position;
                }
                new_op
            }

            // Delete vs Delete
            (
                OperationType::Delete {
                    position: base_pos,
                    length: base_len,
                },
                OperationType::Delete {
                    position: del_pos,
                    length: del_len,
                },
            ) => {
                let new_position = if del_pos < base_pos {
                    base_pos.saturating_sub(del_len.min(base_pos - del_pos))
                } else {
                    *base_pos
                };

                let mut new_op = base_op.clone();
                if let OperationType::Delete { position, .. } = &mut new_op.operation {
                    *position = new_position;
                }
                new_op
            }

            _ => base_op.clone(),
        }
    }

    /// Get current session version
    pub fn get_version(&self, session_id: Uuid) -> Result<u32, String> {
        self.active_sessions
            .get(&session_id)
            .map(|session| session.version)
            .ok_or_else(|| "Session not found".to_string())
    }

    /// Close session and clean up
    pub fn close_session(&self, session_id: Uuid) -> Result<(), String> {
        self.active_sessions.remove(&session_id);
        self.channels.remove(&session_id);
        Ok(())
    }
}

impl Default for CollaborationManager {
    fn default() -> Self {
        *Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let manager = CollaborationManager::new();
        let session_id = Uuid::new_v4();
        let file_id = Uuid::new_v4();

        assert!(manager.create_session(session_id, file_id).is_ok());
        assert!(manager.create_session(session_id, file_id).is_err());
    }

    #[test]
    fn test_join_leave() {
        let manager = CollaborationManager::new();
        let session_id = Uuid::new_v4();
        let file_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        manager.create_session(session_id, file_id).unwrap();
        assert!(manager.join_session(session_id, user_id).is_ok());

        let participants = manager.get_participants(session_id).unwrap();
        assert_eq!(participants.len(), 1);

        assert!(manager.leave_session(session_id, user_id).is_ok());
        let participants = manager.get_participants(session_id).unwrap();
        assert_eq!(participants.len(), 0);
    }

    #[test]
    fn test_cursor_update() {
        let manager = CollaborationManager::new();
        let session_id = Uuid::new_v4();
        let file_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        manager.create_session(session_id, file_id).unwrap();
        manager.join_session(session_id, user_id).unwrap();

        let cursor_update = CursorUpdate {
            user_id,
            session_id,
            cursor_position: 42,
            selection_start: Some(40),
            selection_end: Some(50),
        };

        assert!(manager.update_cursor(session_id, cursor_update).is_ok());
    }
}
