use tracing::error;

use crate::error::{AppError, AppResult};

/// If a task is fail fast after encounter an error node goes down.
pub type IsFailFast = bool;
pub type Task = (IsFailFast, futures::future::BoxFuture<'static, AppResult>);

/// Run the tasks and wait for the process's reason to exit.
///
/// Returns the error of the first fail-fast task that fails. Returns `Ok`
/// once every fail-fast task has finished without one, which is how a
/// graceful shutdown ends: the server task returns `Ok` when its drain
/// completes.
pub async fn join_all(tasks: Vec<Task>) -> AppResult {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<AppError>(1);
    for (is_fail_fast, task) in tasks {
        let sender = if is_fail_fast {
            Some(sender.clone())
        } else {
            None
        };
        tokio::spawn(async {
            if let Err(e) = task.await {
                if let Some(sender) = sender {
                    // The receiver is gone only if join_all already returned.
                    let _ = sender.send(e).await;
                } else {
                    error!("A task failed: {e}.");
                }
            }
        });
    }
    // Only the tasks hold senders now, so the channel closes when the last
    // fail-fast task finishes. Holding this one open made `recv` wait forever
    // after a clean shutdown, and systemd killed the process at
    // TimeoutStopSec on every stop and restart.
    drop(sender);
    match receiver.recv().await {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use std::time::Duration;

    /// The server task returns `Ok` after a graceful shutdown. `join_all`
    /// must return then, or the process never exits and systemd kills it
    /// when `TimeoutStopSec` runs out.
    #[tokio::test]
    async fn returns_when_a_fail_fast_task_finishes_cleanly() {
        let tasks: Vec<Task> = vec![(true, async { Ok(()) }.boxed())];
        let joined = tokio::time::timeout(Duration::from_secs(2), join_all(tasks)).await;
        assert!(
            matches!(joined, Ok(Ok(()))),
            "join_all did not return after its only task finished"
        );
    }

    #[tokio::test]
    async fn returns_the_error_of_a_fail_fast_task() {
        let tasks: Vec<Task> = vec![
            (true, std::future::pending().boxed()),
            (
                true,
                async { Err(AppError::BadRequestError("boom".into())) }.boxed(),
            ),
        ];
        let joined = tokio::time::timeout(Duration::from_secs(2), join_all(tasks)).await;
        assert!(matches!(joined, Ok(Err(AppError::BadRequestError(_)))));
    }

    /// A failure in a task that is not fail-fast is logged and does not end
    /// the process while a fail-fast task is still running.
    #[tokio::test]
    async fn keeps_waiting_after_a_non_fail_fast_failure() {
        let tasks: Vec<Task> = vec![
            (true, std::future::pending().boxed()),
            (
                false,
                async { Err(AppError::BadRequestError("ignored".into())) }.boxed(),
            ),
        ];
        let joined = tokio::time::timeout(Duration::from_millis(200), join_all(tasks)).await;
        assert!(
            joined.is_err(),
            "join_all returned while a task was running"
        );
    }
}
