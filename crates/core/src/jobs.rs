//! The durable job queue: explicit states, retries, failure visibility.
//!
//! Jobs are rows in the `jobs` table (migration v2) driven through an
//! explicit state machine — `queued`, `running`, `succeeded`, `failed`,
//! `cancelled` — with the closed set enforced by a table constraint as well
//! as the Rust type. Every operation validates the transition it performs:
//! completing a job that is not running, failing a job that is not running,
//! or touching an unknown id is a job-queue error, never a silent no-op
//! (A16). The caller supplies the current time so tests and workers stay
//! deterministic; the queue never reads a clock itself.
//!
//! Failure visibility: attempts are counted, the last error text is stored,
//! and failed jobs stay in the table (rows are never deleted), so the UI can
//! always show what failed and why. Cancellation is supported exactly where
//! the reference demands it: a queued job can be cancelled; a running job
//! cannot (the operation is already in flight).

use rusqlite::{Connection, params};

use crate::error::{Error, Result};

/// The explicit states a job moves through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobState {
    /// The state name as stored in the database.
    pub const fn as_str(self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Succeeded => "succeeded",
            JobState::Failed => "failed",
            JobState::Cancelled => "cancelled",
        }
    }

    /// Parses a state name; unknown names map to `None`.
    pub fn from_name(name: &str) -> Option<JobState> {
        match name {
            "queued" => Some(JobState::Queued),
            "running" => Some(JobState::Running),
            "succeeded" => Some(JobState::Succeeded),
            "failed" => Some(JobState::Failed),
            "cancelled" => Some(JobState::Cancelled),
            _ => None,
        }
    }
}

/// How failing attempts are retried before a job becomes `failed`.
pub struct RetryPolicy {
    /// The total number of attempts a job may consume.
    pub max_attempts: i64,
    /// Seconds to wait after a failed attempt before the job is queued again.
    pub backoff_seconds: i64,
}

/// A queued unit of background work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: i64,
    pub kind: String,
    pub payload: String,
    pub state: JobState,
    pub attempts: i64,
    /// The earliest time the job may be claimed, in unix seconds.
    pub run_at: i64,
    pub last_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Enqueues a job of `kind` with `payload`, runnable immediately at `now`.
pub fn enqueue(conn: &Connection, kind: &str, payload: &str, now: i64) -> Result<Job> {
    if kind.trim().is_empty() {
        return Err(Error::InvalidInput("the job kind is empty".into()));
    }
    conn.execute(
        "INSERT INTO jobs (kind, payload, state, attempts, run_at, created_at, updated_at)
         VALUES (?1, ?2, 'queued', 0, ?3, ?3, ?3)",
        params![kind, payload, now],
    )
    .map_err(|e| Error::JobQueue(format!("cannot enqueue the {kind} job: {e}")))?;
    let id = conn.last_insert_rowid();
    get(conn, id)?.ok_or_else(|| Error::JobQueue("the enqueued job cannot be read back".into()))
}

/// Claims the oldest runnable job (run-at time reached) and marks it
/// `running`, or returns `None` when nothing is runnable.
pub fn claim_next(conn: &mut Connection, now: i64) -> Result<Option<Job>> {
    let tx = conn
        .transaction()
        .map_err(|e| Error::JobQueue(format!("cannot start the claim transaction: {e}")))?;
    let raw = match tx.query_row(
        "SELECT id, kind, payload, state, attempts, run_at, last_error, created_at, updated_at
         FROM jobs
         WHERE state = 'queued' AND run_at <= ?1
         ORDER BY run_at, id
         LIMIT 1",
        params![now],
        raw_job_from_row,
    ) {
        Ok(raw) => raw,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            tx.commit().map_err(|e| {
                Error::JobQueue(format!("cannot commit the claim transaction: {e}"))
            })?;
            return Ok(None);
        }
        Err(e) => {
            return Err(Error::JobQueue(format!("cannot find a runnable job: {e}")));
        }
    };

    tx.execute(
        "UPDATE jobs SET state = 'running', updated_at = ?1 WHERE id = ?2",
        params![now, raw.id],
    )
    .map_err(|e| Error::JobQueue(format!("cannot claim job {}: {e}", raw.id)))?;
    tx.commit()
        .map_err(|e| Error::JobQueue(format!("cannot commit the claim of job {}: {e}", raw.id)))?;

    let mut job = raw.into_job()?;
    job.state = JobState::Running;
    job.updated_at = now;
    Ok(Some(job))
}

/// Marks a `running` job `succeeded`.
pub fn complete(conn: &Connection, id: i64, now: i64) -> Result<()> {
    let changed = conn
        .execute(
            "UPDATE jobs SET state = 'succeeded', updated_at = ?1, last_error = NULL
             WHERE id = ?2 AND state = 'running'",
            params![now, id],
        )
        .map_err(|e| Error::JobQueue(format!("cannot complete job {id}: {e}")))?;
    if changed != 1 {
        return Err(not_running(conn, id, "complete"));
    }
    Ok(())
}

/// Records a failed attempt: the job is requeued with a backoff while the
/// attempt budget lasts, and becomes terminally `failed` afterwards.
///
/// Returns the state the job ended in.
pub fn fail_attempt(
    conn: &mut Connection,
    id: i64,
    reason: &str,
    now: i64,
    policy: &RetryPolicy,
) -> Result<JobState> {
    if policy.max_attempts < 1 {
        return Err(Error::InvalidInput(
            "the retry policy must allow at least one attempt".into(),
        ));
    }
    let tx = conn
        .transaction()
        .map_err(|e| Error::JobQueue(format!("cannot start the failure transaction: {e}")))?;
    let raw = tx
        .query_row(
            "SELECT id, kind, payload, state, attempts, run_at, last_error, created_at, updated_at
             FROM jobs WHERE id = ?1",
            params![id],
            raw_job_from_row,
        )
        .map_err(|e| Error::JobQueue(format!("cannot read job {id} to fail it: {e}")))?;
    if raw.state != "running" {
        return Err(Error::JobQueue(format!(
            "job {id} is {} and cannot fail an attempt",
            raw.state
        )));
    }

    let attempts = raw.attempts + 1;
    let final_state = if attempts >= policy.max_attempts {
        "failed"
    } else {
        "queued"
    };
    let run_at = if final_state == "queued" {
        now + policy.backoff_seconds
    } else {
        raw.run_at
    };
    tx.execute(
        "UPDATE jobs SET state = ?1, attempts = ?2, run_at = ?3, last_error = ?4, updated_at = ?5
         WHERE id = ?6",
        params![final_state, attempts, run_at, reason, now, id],
    )
    .map_err(|e| Error::JobQueue(format!("cannot record the failure of job {id}: {e}")))?;
    tx.commit()
        .map_err(|e| Error::JobQueue(format!("cannot commit the failure of job {id}: {e}")))?;

    JobState::from_name(final_state)
        .ok_or_else(|| Error::JobQueue("the computed final state is not a state name".into()))
}

/// Cancels a `queued` job. Returns whether the cancellation happened.
///
/// Running or finished jobs are returned as `false`: the operation is
/// already in flight or already decided (SPEC 10: cancellation where
/// supported).
pub fn cancel(conn: &Connection, id: i64, now: i64) -> Result<bool> {
    let raw = read_raw(conn, id)?;
    if raw.state != "queued" {
        return Ok(false);
    }
    let changed = conn
        .execute(
            "UPDATE jobs SET state = 'cancelled', updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )
        .map_err(|e| Error::JobQueue(format!("cannot cancel job {id}: {e}")))?;
    if changed != 1 {
        return Err(Error::JobQueue(format!(
            "cancelling job {id} changed {changed} rows"
        )));
    }
    Ok(true)
}

/// Reads one job; `None` when the id is unknown.
pub fn get(conn: &Connection, id: i64) -> Result<Option<Job>> {
    read_raw(conn, id).map(|raw| raw.into_job().map(Some))?
}

/// Lists every job in the given state, oldest first.
pub fn list_by_state(conn: &Connection, state: JobState) -> Result<Vec<Job>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, payload, state, attempts, run_at, last_error, created_at, updated_at
             FROM jobs WHERE state = ?1 ORDER BY run_at, id",
        )
        .map_err(|e| Error::JobQueue(format!("cannot list the {} jobs: {e}", state.as_str())))?;
    let rows = stmt
        .query_map(params![state.as_str()], raw_job_from_row)
        .map_err(|e| Error::JobQueue(format!("cannot list the {} jobs: {e}", state.as_str())))?;

    let mut jobs = Vec::new();
    for row in rows {
        let raw = row.map_err(|e| Error::JobQueue(format!("cannot read a job row: {e}")))?;
        jobs.push(raw.into_job()?);
    }
    Ok(jobs)
}

/// A job row as read from the database, before the state is parsed.
struct RawJob {
    id: i64,
    kind: String,
    payload: String,
    state: String,
    attempts: i64,
    run_at: i64,
    last_error: Option<String>,
    created_at: i64,
    updated_at: i64,
}

impl RawJob {
    fn into_job(self) -> Result<Job> {
        let state = JobState::from_name(&self.state).ok_or_else(|| {
            Error::JobQueue(format!(
                "job {} holds the unknown state {}",
                self.id, self.state
            ))
        })?;
        Ok(Job {
            id: self.id,
            kind: self.kind,
            payload: self.payload,
            state,
            attempts: self.attempts,
            run_at: self.run_at,
            last_error: self.last_error,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn raw_job_from_row(row: &rusqlite::Row) -> rusqlite::Result<RawJob> {
    Ok(RawJob {
        id: row.get(0)?,
        kind: row.get(1)?,
        payload: row.get(2)?,
        state: row.get(3)?,
        attempts: row.get(4)?,
        run_at: row.get(5)?,
        last_error: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn read_raw(conn: &Connection, id: i64) -> Result<RawJob> {
    conn.query_row(
        "SELECT id, kind, payload, state, attempts, run_at, last_error, created_at, updated_at
         FROM jobs WHERE id = ?1",
        params![id],
        raw_job_from_row,
    )
    .map_err(|e| {
        if let rusqlite::Error::QueryReturnedNoRows = e {
            Error::JobQueue(format!("job {id} does not exist"))
        } else {
            Error::JobQueue(format!("cannot read job {id}: {e}"))
        }
    })
}

fn not_running(conn: &Connection, id: i64, action: &str) -> Error {
    match read_raw(conn, id) {
        Ok(raw) => Error::JobQueue(format!(
            "job {id} is {} and cannot be marked finished by {action}",
            raw.state
        )),
        Err(Error::JobQueue(message)) if message.contains("does not exist") => {
            Error::JobQueue(message)
        }
        Err(_) => Error::JobQueue(format!("job {id} cannot be found to {action}")),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db;
    use crate::error::Error;

    use super::{
        JobState, RetryPolicy, cancel, claim_next, complete, enqueue, fail_attempt, get,
        list_by_state,
    };

    const NOW: i64 = 1_000;

    fn test_conn() -> Connection {
        let mut conn = Connection::open_in_memory().expect("an in-memory database should open");
        db::run_migrations(&mut conn, db::MIGRATIONS)
            .expect("the migrations should apply to the test database");
        conn
    }

    fn three_attempts() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 3,
            backoff_seconds: 5,
        }
    }

    #[test]
    fn enqueue_stores_a_queued_job() {
        let conn = &mut test_conn();
        let job = enqueue(conn, "sync.inbox", "{}", NOW).expect("the enqueue should succeed");
        assert_eq!(job.id, 1);
        assert_eq!(job.kind, "sync.inbox");
        assert_eq!(job.state, JobState::Queued);
        assert_eq!(job.attempts, 0);
        assert_eq!(job.run_at, NOW);
        assert_eq!(job.last_error, None);

        let read = get(conn, job.id).expect("the job should read back");
        assert_eq!(read, Some(job));
    }

    #[test]
    fn an_empty_job_kind_is_rejected() {
        let conn = &mut test_conn();
        match enqueue(conn, "  ", "{}", NOW) {
            Err(Error::InvalidInput(message)) => {
                assert!(message.contains("kind is empty"), "unexpected: {message}");
            }
            other => panic!("an empty kind must be rejected, got {other:?}"),
        }
    }

    #[test]
    fn claiming_respects_run_at_and_order() {
        let conn = &mut test_conn();
        enqueue(conn, "index.rebuild", "first", NOW).expect("the first enqueue should succeed");
        enqueue(conn, "index.rebuild", "second", NOW + 100)
            .expect("the second enqueue should succeed");

        let first = claim_next(conn, NOW + 50)
            .expect("the first claim should succeed")
            .expect("the first job should be runnable");
        assert_eq!(first.payload, "first");
        assert_eq!(first.state, JobState::Running);

        assert!(
            claim_next(conn, NOW + 50)
                .expect("the second claim should succeed")
                .is_none(),
            "the second job is not runnable yet"
        );

        let second = claim_next(conn, NOW + 100)
            .expect("the third claim should succeed")
            .expect("the second job should be runnable");
        assert_eq!(second.payload, "second");
    }

    #[test]
    fn completing_requires_the_running_state() {
        let conn = &mut test_conn();
        let job = enqueue(conn, "sync.inbox", "{}", NOW).expect("the enqueue should succeed");

        match complete(conn, job.id, NOW + 1) {
            Err(Error::JobQueue(message)) => {
                assert!(
                    message.contains("cannot be marked finished"),
                    "unexpected: {message}"
                );
            }
            other => panic!("completing a queued job must fail, got {other:?}"),
        }

        let claimed = claim_next(conn, NOW).expect("the claim should succeed");
        assert_eq!(claimed.map(|j| j.id), Some(job.id));

        complete(conn, job.id, NOW + 2).expect("the completion should succeed");
        let finished = get(conn, job.id).expect("the job should read back");
        assert_eq!(finished.expect("the job exists").state, JobState::Succeeded);

        match complete(conn, job.id, NOW + 3) {
            Err(Error::JobQueue(message)) => {
                assert!(
                    message.contains("cannot be marked finished"),
                    "unexpected: {message}"
                );
            }
            other => panic!("completing a finished job must fail, got {other:?}"),
        }
    }

    #[test]
    fn failing_requeues_with_backoff_then_gives_up() {
        let conn = &mut test_conn();
        let job = enqueue(conn, "sync.inbox", "{}", NOW).expect("the enqueue should succeed");
        let policy = three_attempts();

        // First failure: back to queued with a backoff.
        claim_next(conn, NOW).expect("the first claim should succeed");
        let state = fail_attempt(conn, job.id, "provider unreachable", NOW + 10, &policy)
            .expect("the first failure should be recorded");
        assert_eq!(state, JobState::Queued);
        let requeued = get(conn, job.id)
            .expect("the job should read back")
            .unwrap();
        assert_eq!(requeued.attempts, 1);
        assert_eq!(requeued.run_at, NOW + 10 + 5);
        assert_eq!(
            requeued.last_error,
            Some("provider unreachable".to_string())
        );

        // Not runnable before the backoff elapses.
        assert!(
            claim_next(conn, NOW + 14)
                .expect("the early claim should succeed")
                .is_none(),
            "the backoff must hold"
        );

        // Second failure: requeued again.
        claim_next(conn, NOW + 15).expect("the second claim should succeed");
        let state = fail_attempt(conn, job.id, "still unreachable", NOW + 20, &policy)
            .expect("the second failure should be recorded");
        assert_eq!(state, JobState::Queued);

        // Third failure: the budget is spent, the job is terminally failed.
        claim_next(conn, NOW + 25).expect("the third claim should succeed");
        let state = fail_attempt(conn, job.id, "giving up", NOW + 30, &policy)
            .expect("the third failure should be recorded");
        assert_eq!(state, JobState::Failed);
        let failed = get(conn, job.id)
            .expect("the job should read back")
            .unwrap();
        assert_eq!(failed.attempts, 3);
        assert_eq!(failed.last_error, Some("giving up".to_string()));

        assert!(
            claim_next(conn, NOW + 10_000)
                .expect("the late claim should succeed")
                .is_none(),
            "a failed job must not run again"
        );
        match fail_attempt(conn, job.id, "late", NOW + 40, &policy) {
            Err(Error::JobQueue(message)) => {
                assert!(
                    message.contains("cannot fail an attempt"),
                    "unexpected: {message}"
                );
            }
            other => panic!("failing a failed job must fail, got {other:?}"),
        }
    }

    #[test]
    fn cancellation_only_touches_queued_jobs() {
        let conn = &mut test_conn();
        let first =
            enqueue(conn, "sync.inbox", "one", NOW).expect("the first enqueue should succeed");
        let second =
            enqueue(conn, "sync.inbox", "two", NOW).expect("the second enqueue should succeed");
        // The oldest runnable job is claimed first: `first` becomes running.
        claim_next(conn, NOW).expect("the claim should pick the first job");

        assert!(
            !cancel(conn, first.id, NOW + 1).expect("the cancel call should answer"),
            "a running job must not be cancellable"
        );
        let unchanged = get(conn, first.id).expect("the running job should read back");
        assert_eq!(unchanged.expect("the job exists").state, JobState::Running);

        assert!(
            cancel(conn, second.id, NOW + 3).expect("cancelling the queued job should answer"),
            "a queued job should be cancellable"
        );
        let cancelled = get(conn, second.id).expect("the cancelled job should read back");
        assert_eq!(
            cancelled.expect("the job exists").state,
            JobState::Cancelled
        );

        assert!(
            !cancel(conn, second.id, NOW + 4).expect("the second cancel should answer"),
            "an already cancelled job reports no new cancellation"
        );

        match cancel(conn, 999, NOW + 5) {
            Err(Error::JobQueue(message)) => {
                assert!(message.contains("does not exist"), "unexpected: {message}");
            }
            other => panic!("an unknown job must be an error, got {other:?}"),
        }
    }

    #[test]
    fn lists_are_per_state_and_ordered() {
        let conn = &mut test_conn();
        enqueue(conn, "sync.inbox", "one", NOW).expect("the first enqueue should succeed");
        enqueue(conn, "sync.inbox", "two", NOW).expect("the second enqueue should succeed");
        claim_next(conn, NOW).expect("the first job should be claimed");

        let queued = list_by_state(conn, JobState::Queued).expect("the queued list should work");
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].payload, "two");

        let running = list_by_state(conn, JobState::Running).expect("the running list should work");
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].payload, "one");

        let succeeded =
            list_by_state(conn, JobState::Succeeded).expect("the succeeded list should work");
        assert_eq!(succeeded.len(), 0);
    }

    #[test]
    fn the_database_rejects_unknown_states() {
        let conn = &mut test_conn();
        let result = conn.execute(
            "INSERT INTO jobs (kind, payload, state, attempts, run_at, created_at, updated_at)
             VALUES ('probe.kind', '{}', 'bogus', 0, 0, 0, 0)",
            [],
        );
        assert!(
            result.is_err(),
            "the CHECK constraint must keep unknown states out of the jobs table"
        );
    }
}
