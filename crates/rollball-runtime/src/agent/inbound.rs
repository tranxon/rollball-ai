//! Inbound message queue for external message injection
//!
//! Allows external sources (Gateway, cross-agent intents, system notifications)
//! to inject messages into the agent loop without blocking the main execution.

use serde::{Deserialize, Serialize};

/// Messages that can be injected into the agent loop from external sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InboundMessage {
    /// Direct user message
    UserMessage(String),
    /// System notification (identity update, capability change, etc.)
    SystemNotification {
        notification_type: String,
        data: serde_json::Value,
    },
    /// Cross-agent intent message
    IntentMessage {
        from: String,
        action: String,
        params: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inbound_message_serde_roundtrip() {
        let msg = InboundMessage::IntentMessage {
            from: "com.rollball.system".to_string(),
            action: "update_identity".to_string(),
            params: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: InboundMessage = serde_json::from_str(&json).unwrap();
        if let InboundMessage::IntentMessage { from, action, params } = decoded {
            assert_eq!(from, "com.rollball.system");
            assert_eq!(action, "update_identity");
            assert_eq!(params["key"], "value");
        } else {
            panic!("Expected IntentMessage");
        }
    }
}
