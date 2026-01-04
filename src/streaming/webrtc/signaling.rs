//! WebRTC signaling protocol for real-time video streaming.
//!
//! This module defines the WebSocket-based signaling protocol used for WebRTC
//! negotiation between clients and the server. It handles SDP (Session Description
//! Protocol) exchange and ICE (Interactive Connectivity Establishment) candidate
//! exchange required for establishing peer-to-peer connections.
//!
//! # Protocol Flow
//!
//! 1. **Offer/Answer Exchange**: Client sends SDP offer, server responds with answer
//! 2. **ICE Candidate Exchange**: Both parties exchange ICE candidates for connectivity
//! 3. **Connection Establishment**: Once candidates are exchanged, WebRTC connection is established
//! 4. **Stats/Monitoring**: Client can request connection statistics
//! 5. **Hangup**: Either party can terminate the session
//!
//! # Example Client Flow
//!
//! ```json
//! // 1. Client sends offer
//! {
//!   "type": "offer",
//!   "session_id": null,
//!   "sdp": "v=0\r\no=- 123456789 2 IN IP4..."
//! }
//!
//! // 2. Server responds with answer
//! {
//!   "type": "answer",
//!   "session_id": "abc-123-def",
//!   "sdp": "v=0\r\no=- 987654321 2 IN IP4..."
//! }
//!
//! // 3. Both parties exchange ICE candidates
//! {
//!   "type": "ice_candidate",
//!   "session_id": "abc-123-def",
//!   "candidate": "candidate:1 1 UDP 2130706431...",
//!   "sdp_mid": "0",
//!   "sdp_mline_index": 0
//! }
//!
//! // 4. Server confirms connection
//! {
//!   "type": "connected",
//!   "session_id": "abc-123-def",
//!   "monitor_id": 1
//! }
//! ```

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Client to server signaling messages.
///
/// These messages are sent from the WebRTC client to the server over a WebSocket
/// connection to negotiate the connection parameters and exchange ICE candidates.
///
/// # Message Types
///
/// - **Offer**: Initiates a WebRTC session with an SDP offer
/// - **Answer**: Responds to a server-initiated offer (less common)
/// - **IceCandidate**: Sends an ICE candidate for connectivity establishment
/// - **Hangup**: Requests session termination
/// - **GetStats**: Requests current connection statistics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Client sends SDP offer to initiate connection.
    ///
    /// The offer contains the session description that includes media capabilities,
    /// codecs, and connection information. The `session_id` is optional for the
    /// initial offer and will be assigned by the server.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "offer",
    ///   "session_id": null,
    ///   "sdp": "v=0\r\no=- 4611731400430051336 2 IN IP4 127.0.0.1..."
    /// }
    /// ```
    Offer {
        /// Optional session ID (null for initial offer, assigned by server)
        session_id: Option<String>,
        /// SDP offer in string format
        sdp: String,
    },

    /// Client sends SDP answer in response to server offer.
    ///
    /// This is used when the server initiates the connection (less common in
    /// typical WebRTC flows). The answer must include the session_id provided
    /// by the server in its offer.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "answer",
    ///   "session_id": "abc-123-def",
    ///   "sdp": "v=0\r\no=- 4611731400430051336 2 IN IP4 127.0.0.1..."
    /// }
    /// ```
    Answer {
        /// Session ID from the server's offer
        session_id: String,
        /// SDP answer in string format
        sdp: String,
    },

    /// Client sends ICE candidate for connectivity check.
    ///
    /// ICE candidates represent possible network paths for establishing the
    /// peer connection. Multiple candidates may be sent as they are discovered.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "ice_candidate",
    ///   "session_id": "abc-123-def",
    ///   "candidate": "candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host",
    ///   "sdp_mid": "0",
    ///   "sdp_mline_index": 0
    /// }
    /// ```
    IceCandidate {
        /// Session ID for this connection
        session_id: String,
        /// ICE candidate string
        candidate: String,
        /// Media stream identification tag
        sdp_mid: Option<String>,
        /// Media line index in the SDP
        sdp_mline_index: Option<u16>,
    },

    /// Client requests to end the session.
    ///
    /// Gracefully terminates the WebRTC session and releases resources.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "hangup",
    ///   "session_id": "abc-123-def"
    /// }
    /// ```
    Hangup {
        /// Session ID to terminate
        session_id: String,
    },

    /// Client requests current stats.
    ///
    /// Requests real-time statistics about the connection such as bitrate,
    /// packet loss, jitter, etc.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "get_stats",
    ///   "session_id": "abc-123-def"
    /// }
    /// ```
    GetStats {
        /// Session ID to get stats for
        session_id: String,
    },
}

/// Server to client signaling messages.
///
/// These messages are sent from the server to the WebRTC client to respond to
/// client requests, provide ICE candidates, and notify about connection status.
///
/// # Message Types
///
/// - **Offer**: Server-initiated SDP offer (less common)
/// - **Answer**: Response to client's offer with SDP answer
/// - **IceCandidate**: Server's ICE candidate for connectivity
/// - **Connected**: Notification that connection is established
/// - **Disconnected**: Notification that connection was closed
/// - **Error**: Error occurred during signaling or connection
/// - **Stats**: Response to GetStats request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Server sends SDP offer (if server initiates).
    ///
    /// Used when the server needs to initiate the WebRTC connection. Less common
    /// than client-initiated flows but useful for certain scenarios.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "offer",
    ///   "session_id": "abc-123-def",
    ///   "sdp": "v=0\r\no=- 987654321 2 IN IP4 10.0.0.1..."
    /// }
    /// ```
    Offer {
        /// Newly assigned session ID
        session_id: String,
        /// SDP offer in string format
        sdp: String,
    },

    /// Server sends SDP answer in response to client offer.
    ///
    /// This is the most common server response, containing the server's session
    /// description in response to the client's offer.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "answer",
    ///   "session_id": "abc-123-def",
    ///   "sdp": "v=0\r\no=- 987654321 2 IN IP4 10.0.0.1..."
    /// }
    /// ```
    Answer {
        /// Session ID for this connection
        session_id: String,
        /// SDP answer in string format
        sdp: String,
    },

    /// Server sends ICE candidate.
    ///
    /// The server sends its discovered ICE candidates to the client for
    /// establishing connectivity.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "ice_candidate",
    ///   "session_id": "abc-123-def",
    ///   "candidate": "candidate:1 1 UDP 2130706431 10.0.0.1 12345 typ host",
    ///   "sdp_mid": "0",
    ///   "sdp_mline_index": 0
    /// }
    /// ```
    IceCandidate {
        /// Session ID for this connection
        session_id: String,
        /// ICE candidate string
        candidate: String,
        /// Media stream identification tag
        sdp_mid: Option<String>,
        /// Media line index in the SDP
        sdp_mline_index: Option<u16>,
    },

    /// Connection established successfully.
    ///
    /// Sent when the WebRTC connection is fully established and ready to
    /// stream video data.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "connected",
    ///   "session_id": "abc-123-def",
    ///   "monitor_id": 1
    /// }
    /// ```
    Connected {
        /// Session ID for this connection
        session_id: String,
        /// Monitor ID being streamed
        monitor_id: u32,
    },

    /// Connection was closed.
    ///
    /// Notifies the client that the connection has been terminated, either
    /// due to a client request, server decision, or error condition.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "disconnected",
    ///   "session_id": "abc-123-def",
    ///   "reason": "Client requested hangup"
    /// }
    /// ```
    Disconnected {
        /// Session ID that was disconnected
        session_id: String,
        /// Human-readable reason for disconnection
        reason: String,
    },

    /// Error occurred.
    ///
    /// Sent when an error occurs during signaling, negotiation, or connection.
    /// The `code` field contains a machine-readable error code from the
    /// `error_codes` module.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "error",
    ///   "session_id": "abc-123-def",
    ///   "code": "SESSION_NOT_FOUND",
    ///   "message": "The requested session does not exist"
    /// }
    /// ```
    Error {
        /// Session ID if applicable (null for session-independent errors)
        session_id: Option<String>,
        /// Machine-readable error code (see `error_codes` module)
        code: String,
        /// Human-readable error message
        message: String,
    },

    /// Statistics response.
    ///
    /// Contains real-time statistics about the WebRTC connection including
    /// bitrate, packet loss, jitter, round-trip time, etc.
    ///
    /// # Example
    ///
    /// ```json
    /// {
    ///   "type": "stats",
    ///   "session_id": "abc-123-def",
    ///   "stats": {
    ///     "bitrate": 2500000,
    ///     "packet_loss": 0.01,
    ///     "jitter": 5.2,
    ///     "rtt": 45
    ///   }
    /// }
    /// ```
    Stats {
        /// Session ID for these stats
        session_id: String,
        /// Statistics data as JSON object
        stats: serde_json::Value,
    },
}

/// Error codes for signaling errors.
///
/// These constants define standard error codes used in `ServerMessage::Error`
/// messages. They provide machine-readable error identifiers that clients
/// can use for error handling and user feedback.
///
/// # Usage
///
/// ```rust
/// use zm_api::streaming::webrtc::signaling::{ServerMessage, error_codes};
///
/// let error = ServerMessage::error(
///     Some("session-123".to_string()),
///     error_codes::SESSION_NOT_FOUND,
///     "The specified session does not exist"
/// );
/// ```
pub mod error_codes {
    /// Invalid or malformed message received from client.
    ///
    /// Returned when the client sends a message that cannot be parsed or
    /// contains invalid data.
    pub const INVALID_MESSAGE: &str = "INVALID_MESSAGE";

    /// Session ID not found.
    ///
    /// Returned when the client references a session ID that doesn't exist
    /// or has expired.
    pub const SESSION_NOT_FOUND: &str = "SESSION_NOT_FOUND";

    /// Monitor not found or not accessible.
    ///
    /// Returned when the requested monitor ID doesn't exist or the user
    /// doesn't have permission to access it.
    pub const MONITOR_NOT_FOUND: &str = "MONITOR_NOT_FOUND";

    /// Client is not authorized.
    ///
    /// Returned when authentication fails or the client doesn't have
    /// permission to perform the requested operation.
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";

    /// Maximum number of sessions reached.
    ///
    /// Returned when the server has reached its limit of concurrent WebRTC
    /// sessions and cannot accept new connections.
    pub const MAX_SESSIONS: &str = "MAX_SESSIONS";

    /// Internal server error.
    ///
    /// Returned when an unexpected error occurs on the server side during
    /// signaling or connection setup.
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";

    /// ICE connection failed.
    ///
    /// Returned when ICE negotiation fails and a peer connection cannot
    /// be established.
    pub const ICE_FAILED: &str = "ICE_FAILED";
}

impl ServerMessage {
    /// Creates an error message.
    ///
    /// Helper method to construct a `ServerMessage::Error` with the specified
    /// parameters.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Optional session ID if error is session-specific
    /// * `code` - Error code from the `error_codes` module
    /// * `message` - Human-readable error message
    ///
    /// # Example
    ///
    /// ```rust
    /// use zm_api::streaming::webrtc::signaling::{ServerMessage, error_codes};
    ///
    /// let error = ServerMessage::error(
    ///     Some("abc-123".to_string()),
    ///     error_codes::MONITOR_NOT_FOUND,
    ///     "Monitor 999 does not exist"
    /// );
    /// ```
    pub fn error(session_id: Option<String>, code: &str, message: impl Into<String>) -> Self {
        ServerMessage::Error {
            session_id,
            code: code.to_string(),
            message: message.into(),
        }
    }

    /// Serializes the message to JSON string.
    ///
    /// Converts the server message to a JSON string suitable for sending
    /// over a WebSocket connection.
    ///
    /// # Returns
    ///
    /// JSON string representation of the message
    ///
    /// # Example
    ///
    /// ```rust
    /// use zm_api::streaming::webrtc::signaling::ServerMessage;
    ///
    /// let msg = ServerMessage::Connected {
    ///     session_id: "abc-123".to_string(),
    ///     monitor_id: 1,
    /// };
    /// let json = msg.to_json();
    /// // {"type":"connected","session_id":"abc-123","monitor_id":1}
    /// ```
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Fallback to a generic error message if serialization fails
            r#"{"type":"error","session_id":null,"code":"INTERNAL_ERROR","message":"Failed to serialize message"}"#.to_string()
        })
    }
}

impl ClientMessage {
    /// Parses a JSON string into a ClientMessage.
    ///
    /// Deserializes a JSON string received from the client over a WebSocket
    /// into a `ClientMessage` enum variant.
    ///
    /// # Arguments
    ///
    /// * `text` - JSON string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(ClientMessage)` - Successfully parsed message
    /// * `Err(serde_json::Error)` - Parse error with details
    ///
    /// # Example
    ///
    /// ```rust
    /// use zm_api::streaming::webrtc::signaling::ClientMessage;
    ///
    /// let json = r#"{"type":"offer","session_id":null,"sdp":"v=0..."}"#;
    /// match ClientMessage::parse(json) {
    ///     Ok(msg) => println!("Received: {:?}", msg),
    ///     Err(e) => eprintln!("Parse error: {}", e),
    /// }
    /// ```
    pub fn parse(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_offer_serialization() {
        let offer = ClientMessage::Offer {
            session_id: None,
            sdp: "v=0\r\no=- 123 2 IN IP4 127.0.0.1".to_string(),
        };

        let json = serde_json::to_string(&offer).unwrap();
        assert!(json.contains("\"type\":\"offer\""));
        assert!(json.contains("\"session_id\":null"));
    }

    #[test]
    fn test_client_offer_deserialization() {
        let json =
            r#"{"type":"offer","session_id":null,"sdp":"v=0\r\no=- 123 2 IN IP4 127.0.0.1"}"#;
        let msg = ClientMessage::parse(json).unwrap();

        match msg {
            ClientMessage::Offer { session_id, sdp } => {
                assert!(session_id.is_none());
                assert_eq!(sdp, "v=0\r\no=- 123 2 IN IP4 127.0.0.1");
            }
            _ => panic!("Expected Offer variant"),
        }
    }

    #[test]
    fn test_client_ice_candidate_parsing() {
        let json = r#"{
            "type":"ice_candidate",
            "session_id":"abc-123",
            "candidate":"candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host",
            "sdp_mid":"0",
            "sdp_mline_index":0
        }"#;

        let msg = ClientMessage::parse(json).unwrap();

        match msg {
            ClientMessage::IceCandidate {
                session_id,
                candidate,
                sdp_mid,
                sdp_mline_index,
            } => {
                assert_eq!(session_id, "abc-123");
                assert!(candidate.contains("192.168.1.100"));
                assert_eq!(sdp_mid, Some("0".to_string()));
                assert_eq!(sdp_mline_index, Some(0));
            }
            _ => panic!("Expected IceCandidate variant"),
        }
    }

    #[test]
    fn test_server_answer_serialization() {
        let answer = ServerMessage::Answer {
            session_id: "test-session".to_string(),
            sdp: "v=0\r\no=- 456 2 IN IP4 10.0.0.1".to_string(),
        };

        let json = answer.to_json();
        assert!(json.contains("\"type\":\"answer\""));
        assert!(json.contains("\"session_id\":\"test-session\""));
    }

    #[test]
    fn test_server_error_helper() {
        let error = ServerMessage::error(
            Some("session-123".to_string()),
            error_codes::SESSION_NOT_FOUND,
            "Session does not exist",
        );

        match error {
            ServerMessage::Error {
                session_id,
                code,
                message,
            } => {
                assert_eq!(session_id, Some("session-123".to_string()));
                assert_eq!(code, "SESSION_NOT_FOUND");
                assert_eq!(message, "Session does not exist");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_server_connected_message() {
        let connected = ServerMessage::Connected {
            session_id: "abc-123".to_string(),
            monitor_id: 42,
        };

        let json = connected.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "connected");
        assert_eq!(parsed["session_id"], "abc-123");
        assert_eq!(parsed["monitor_id"], 42);
    }

    #[test]
    fn test_client_hangup_parsing() {
        let json = r#"{"type":"hangup","session_id":"test-123"}"#;
        let msg = ClientMessage::parse(json).unwrap();

        match msg {
            ClientMessage::Hangup { session_id } => {
                assert_eq!(session_id, "test-123");
            }
            _ => panic!("Expected Hangup variant"),
        }
    }

    #[test]
    fn test_server_stats_message() {
        let stats_data = serde_json::json!({
            "bitrate": 2500000,
            "packet_loss": 0.01,
            "jitter": 5.2
        });

        let stats = ServerMessage::Stats {
            session_id: "stats-session".to_string(),
            stats: stats_data,
        };

        let json = stats.to_json();
        assert!(json.contains("\"bitrate\":2500000"));
    }

    #[test]
    fn test_invalid_message_parsing() {
        let json = r#"{"type":"invalid_type"}"#;
        let result = ClientMessage::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_codes_constants() {
        assert_eq!(error_codes::INVALID_MESSAGE, "INVALID_MESSAGE");
        assert_eq!(error_codes::SESSION_NOT_FOUND, "SESSION_NOT_FOUND");
        assert_eq!(error_codes::MONITOR_NOT_FOUND, "MONITOR_NOT_FOUND");
        assert_eq!(error_codes::UNAUTHORIZED, "UNAUTHORIZED");
        assert_eq!(error_codes::MAX_SESSIONS, "MAX_SESSIONS");
        assert_eq!(error_codes::INTERNAL_ERROR, "INTERNAL_ERROR");
        assert_eq!(error_codes::ICE_FAILED, "ICE_FAILED");
    }
}
