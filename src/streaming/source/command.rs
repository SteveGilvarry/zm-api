//! Request/response broker for zm-next on-demand worker commands.
//!
//! zm-api sends a `0x11 Command` (`snapshot_now`, `describe_now`) on a
//! monitor's stream-socket connection. The worker answers twice, in either
//! order:
//!
//! * a `0x12 Response` to this connection only: `ok:true` means "dispatched to
//!   the plugins", `ok:false` means rejected (e.g. `unknown_command`);
//! * one result EVENT, fanned out to every connected client, whose JSON detail
//!   carries the same `request_id` plus `on_demand:true` and `ok`.
//!
//! If no plugin owns the command, only the Response arrives, so every request
//! has a timeout. Requests are tied to the connection they were sent on and
//! fail as soon as that connection drops (worker restart, reader stopped).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::oneshot;

use super::protocol::{CommandResponse, MonitorEvent, EVENT_DESCRIPTION, EVENT_SNAPSHOT_SAVED};

/// Why an on-demand command produced no result.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The monitor has no live worker connection to send on.
    #[error("no worker connection for monitor {0}")]
    NotConnected(u32),
    /// The worker refused the command (its Response had `ok:false`).
    #[error("worker rejected the command: {0}")]
    Rejected(String),
    /// The owning plugin ran the command and reported failure.
    #[error("{0}")]
    Failed(String),
    /// The worker accepted the command but no plugin answered in time. Usually
    /// the pipeline has no plugin for it (no `store_snapshot` / `describe_vlm`).
    #[error("worker accepted the command but no plugin answered within {0:?}; does the pipeline include the plugin for it?")]
    NoResult(Duration),
    /// Nothing at all came back in time.
    #[error("worker did not respond within {0:?}")]
    Timeout(Duration),
    /// The connection closed before the result arrived.
    #[error("worker connection closed before the result arrived")]
    Disconnected,
}

/// Outcome delivered to a waiting request: the result EVENT's JSON detail.
type Outcome = Result<Value, CommandError>;

struct Pending {
    monitor_id: u32,
    /// Connection the command went out on; a drop of that connection fails it.
    conn_id: u64,
    /// The worker's Response said `ok:true`. Only used to word a timeout.
    dispatched: bool,
    tx: oneshot::Sender<Outcome>,
}

/// Shared broker. One per [`super::router::SourceRouter`]; the reader tasks feed
/// it Responses, result EVENTs and connection drops.
pub struct CommandBroker {
    pending: DashMap<u64, Pending>,
    next_request_id: AtomicU64,
    next_conn_id: AtomicU64,
}

impl Default for CommandBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBroker {
    pub fn new() -> Self {
        // Result EVENTs go to every client on the socket, so another client
        // (a second zm-api, wl_dump) may use small request ids of its own.
        // Starting from the clock keeps ours out of their way.
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(1);
        Self {
            pending: DashMap::new(),
            next_request_id: AtomicU64::new(seed),
            next_conn_id: AtomicU64::new(1),
        }
    }

    /// A fresh id for a newly opened connection.
    pub fn connection_id(&self) -> u64 {
        self.next_conn_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Send `command` (a JSON object; `request_id` is filled in here) and wait
    /// for its result. `send` queues the encoded JSON on the connection
    /// identified by `conn_id` and returns whether it was queued.
    pub async fn request(
        &self,
        monitor_id: u32,
        conn_id: u64,
        mut command: Value,
        timeout: Duration,
        send: impl FnOnce(&str) -> bool,
    ) -> Outcome {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        command["request_id"] = request_id.into();
        let (tx, rx) = oneshot::channel();
        // Register before sending: the result can beat the Response back.
        self.pending.insert(
            request_id,
            Pending {
                monitor_id,
                conn_id,
                dispatched: false,
                tx,
            },
        );
        // Removes the entry however this future ends, including when the HTTP
        // client goes away and the future is dropped mid-wait.
        let _guard = PendingGuard {
            broker: self,
            request_id,
        };

        if !send(&command.to_string()) {
            return Err(CommandError::NotConnected(monitor_id));
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(outcome)) => outcome,
            // Sender dropped without an outcome; only happens if the entry was
            // removed by something other than a resolve.
            Ok(Err(_)) => Err(CommandError::Disconnected),
            Err(_) => {
                let dispatched = self
                    .pending
                    .get(&request_id)
                    .map(|p| p.dispatched)
                    .unwrap_or(false);
                if dispatched {
                    Err(CommandError::NoResult(timeout))
                } else {
                    Err(CommandError::Timeout(timeout))
                }
            }
        }
    }

    /// Feed a Response received on `conn_id`. A rejection resolves the request
    /// at once; an acceptance is noted and the wait for the result goes on.
    pub fn on_response(&self, conn_id: u64, resp: &CommandResponse) {
        if resp.ok {
            if let Some(mut p) = self.pending.get_mut(&resp.request_id) {
                if p.conn_id == conn_id {
                    p.dispatched = true;
                }
            }
            return;
        }
        if let Some((_, p)) = self
            .pending
            .remove_if(&resp.request_id, |_, p| p.conn_id == conn_id)
        {
            let _ = p.tx.send(Err(CommandError::Rejected(resp.message.clone())));
        }
    }

    /// Feed a result EVENT from `monitor_id`. Resolves the matching request, if
    /// any is waiting.
    pub fn on_event(&self, monitor_id: u32, event: &MonitorEvent) {
        let Some(detail) = on_demand_detail(event) else {
            return;
        };
        let Some(request_id) = detail.get("request_id").and_then(Value::as_u64) else {
            return;
        };
        let Some((_, p)) = self
            .pending
            .remove_if(&request_id, |_, p| p.monitor_id == monitor_id)
        else {
            return;
        };
        let outcome = if detail.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            Ok(detail)
        } else {
            let error = detail
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("the plugin reported a failure without a reason");
            Err(CommandError::Failed(error.to_string()))
        };
        let _ = p.tx.send(outcome);
    }

    /// Fail every request sent on `conn_id`; its connection is gone.
    pub fn on_disconnect(&self, conn_id: u64) {
        let ids: Vec<u64> = self
            .pending
            .iter()
            .filter(|e| e.conn_id == conn_id)
            .map(|e| *e.key())
            .collect();
        for id in ids {
            if let Some((_, p)) = self.pending.remove_if(&id, |_, p| p.conn_id == conn_id) {
                let _ = p.tx.send(Err(CommandError::Disconnected));
            }
        }
    }

    #[cfg(test)]
    fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

struct PendingGuard<'a> {
    broker: &'a CommandBroker,
    request_id: u64,
}

impl Drop for PendingGuard<'_> {
    fn drop(&mut self) {
        self.broker.pending.remove(&self.request_id);
    }
}

/// The JSON detail of an on-demand result EVENT, or `None` for anything else
/// (routine snapshots and descriptions included). Only the two result codes
/// are parsed, so the per-event cost for other traffic is one comparison.
pub fn on_demand_detail(event: &MonitorEvent) -> Option<Value> {
    if event.code != EVENT_SNAPSHOT_SAVED && event.code != EVENT_DESCRIPTION {
        return None;
    }
    let detail: Value = serde_json::from_str(event.json_detail.as_deref()?).ok()?;
    detail
        .get("on_demand")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        .then_some(detail)
}

/// Cheaply shareable handle, as stored on the router and its reader tasks.
pub type SharedBroker = Arc<CommandBroker>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::sync::mpsc;

    const LONG: Duration = Duration::from_secs(5);

    fn result_event(code: u16, detail: Value) -> MonitorEvent {
        MonitorEvent {
            code,
            json_detail: Some(detail.to_string()),
            ..MonitorEvent::default()
        }
    }

    fn response(request_id: u64, ok: bool, message: &str) -> CommandResponse {
        CommandResponse {
            request_id,
            ok,
            message: message.into(),
            data: serde_json::Value::Null,
        }
    }

    /// Start a request in the background. Returns the sent JSON and the
    /// request's eventual outcome.
    fn spawn_request(
        broker: &Arc<CommandBroker>,
        monitor_id: u32,
        conn_id: u64,
        timeout: Duration,
    ) -> (
        mpsc::UnboundedReceiver<Value>,
        tokio::task::JoinHandle<Outcome>,
    ) {
        let (sent_tx, sent_rx) = mpsc::unbounded_channel();
        let b = broker.clone();
        let handle = tokio::spawn(async move {
            b.request(
                monitor_id,
                conn_id,
                json!({"cmd": "snapshot_now"}),
                timeout,
                move |s| sent_tx.send(serde_json::from_str(s).unwrap()).is_ok(),
            )
            .await
        });
        (sent_rx, handle)
    }

    async fn sent_request_id(rx: &mut mpsc::UnboundedReceiver<Value>) -> u64 {
        let sent = rx.recv().await.expect("command sent");
        assert_eq!(sent["cmd"], "snapshot_now");
        sent["request_id"].as_u64().expect("request_id filled in")
    }

    #[tokio::test]
    async fn result_before_response_resolves() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        let id = sent_request_id(&mut sent).await;

        broker.on_event(
            3,
            &result_event(
                EVENT_SNAPSHOT_SAVED,
                json!({"request_id": id, "on_demand": true, "ok": true, "path": "/s/a.jpg"}),
            ),
        );
        // The late Response must not disturb the already-resolved request.
        broker.on_response(1, &response(id, true, "dispatched"));

        let detail = handle.await.unwrap().expect("resolved");
        assert_eq!(detail["path"], "/s/a.jpg");
        assert_eq!(broker.pending_count(), 0);
    }

    #[tokio::test]
    async fn response_then_result_resolves() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        let id = sent_request_id(&mut sent).await;

        broker.on_response(1, &response(id, true, "dispatched"));
        broker.on_event(
            3,
            &result_event(
                EVENT_DESCRIPTION,
                json!({"request_id": id, "on_demand": true, "ok": true, "text": "a cat"}),
            ),
        );

        assert_eq!(handle.await.unwrap().unwrap()["text"], "a cat");
    }

    #[tokio::test]
    async fn rejected_response_fails_at_once() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        let id = sent_request_id(&mut sent).await;

        broker.on_response(1, &response(id, false, "unknown_command: snapshot_now"));

        assert_eq!(
            handle.await.unwrap(),
            Err(CommandError::Rejected(
                "unknown_command: snapshot_now".into()
            ))
        );
    }

    #[tokio::test]
    async fn plugin_failure_carries_its_error() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        let id = sent_request_id(&mut sent).await;

        broker.on_event(
            3,
            &result_event(
                EVENT_DESCRIPTION,
                json!({"request_id": id, "on_demand": true, "ok": false,
                       "error": "VLM server request failed"}),
            ),
        );

        assert_eq!(
            handle.await.unwrap(),
            Err(CommandError::Failed("VLM server request failed".into()))
        );
    }

    #[tokio::test]
    async fn response_only_times_out_as_no_result() {
        let broker = Arc::new(CommandBroker::new());
        let timeout = Duration::from_millis(100);
        let (mut sent, handle) = spawn_request(&broker, 3, 1, timeout);
        let id = sent_request_id(&mut sent).await;

        // The pipeline has no owning plugin: only the Response ever arrives.
        broker.on_response(1, &response(id, true, "dispatched"));

        assert_eq!(handle.await.unwrap(), Err(CommandError::NoResult(timeout)));
        assert_eq!(broker.pending_count(), 0);
    }

    #[tokio::test]
    async fn silence_times_out() {
        let broker = Arc::new(CommandBroker::new());
        let timeout = Duration::from_millis(100);
        let (mut sent, handle) = spawn_request(&broker, 3, 1, timeout);
        sent_request_id(&mut sent).await;

        assert_eq!(handle.await.unwrap(), Err(CommandError::Timeout(timeout)));
        assert_eq!(broker.pending_count(), 0);
    }

    #[tokio::test]
    async fn disconnect_fails_only_that_connections_requests() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent_a, on_a) = spawn_request(&broker, 3, 1, LONG);
        let (mut sent_b, on_b) = spawn_request(&broker, 4, 2, LONG);
        sent_request_id(&mut sent_a).await;
        let id_b = sent_request_id(&mut sent_b).await;

        broker.on_disconnect(1);
        assert_eq!(on_a.await.unwrap(), Err(CommandError::Disconnected));

        // The other monitor's connection is untouched.
        broker.on_event(
            4,
            &result_event(
                EVENT_SNAPSHOT_SAVED,
                json!({"request_id": id_b, "on_demand": true, "ok": true}),
            ),
        );
        assert!(on_b.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn unsent_command_fails_and_cleans_up() {
        let broker = CommandBroker::new();
        let outcome = broker
            .request(3, 1, json!({"cmd": "snapshot_now"}), LONG, |_| false)
            .await;
        assert_eq!(outcome, Err(CommandError::NotConnected(3)));
        assert_eq!(broker.pending_count(), 0);
    }

    #[tokio::test]
    async fn dropped_request_future_cleans_up() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        sent_request_id(&mut sent).await;
        assert_eq!(broker.pending_count(), 1);

        handle.abort();
        let _ = handle.await;
        assert_eq!(broker.pending_count(), 0);
    }

    #[tokio::test]
    async fn results_for_other_monitors_or_ids_are_ignored() {
        let broker = Arc::new(CommandBroker::new());
        let (mut sent, handle) = spawn_request(&broker, 3, 1, LONG);
        let id = sent_request_id(&mut sent).await;

        // Same id from a different monitor's socket, and a foreign id.
        broker.on_event(
            9,
            &result_event(
                EVENT_SNAPSHOT_SAVED,
                json!({"request_id": id, "on_demand": true, "ok": true}),
            ),
        );
        broker.on_event(
            3,
            &result_event(
                EVENT_SNAPSHOT_SAVED,
                json!({"request_id": id + 1000, "on_demand": true, "ok": true}),
            ),
        );
        // A rejection arriving on some other connection.
        broker.on_response(2, &response(id, false, "nope"));
        assert_eq!(broker.pending_count(), 1);

        broker.on_disconnect(1);
        assert_eq!(handle.await.unwrap(), Err(CommandError::Disconnected));
    }

    #[test]
    fn on_demand_detail_only_matches_tagged_results() {
        let tagged = json!({"request_id": 1, "on_demand": true, "ok": true});
        assert!(on_demand_detail(&result_event(EVENT_SNAPSHOT_SAVED, tagged.clone())).is_some());
        assert!(on_demand_detail(&result_event(EVENT_DESCRIPTION, tagged.clone())).is_some());
        // Routine snapshot, other codes, and unparseable detail.
        assert!(on_demand_detail(&result_event(
            EVENT_SNAPSHOT_SAVED,
            json!({"path": "/s/a.jpg"})
        ))
        .is_none());
        assert!(on_demand_detail(&result_event(0x0301, tagged)).is_none());
        let mut bad = result_event(EVENT_DESCRIPTION, json!({}));
        bad.json_detail = Some("{".into());
        assert!(on_demand_detail(&bad).is_none());
    }
}
