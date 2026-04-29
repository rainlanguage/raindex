pub mod environment;
pub mod remotes;
pub mod utils;

use crate::local_db::pipeline::SyncOutcome;
use crate::local_db::{LocalDbError, RaindexIdentifier};

/// Stage at which a target can fail during a runner invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetStage {
    ManifestFetch,
    DumpDownload,
    EngineBuild,
    EngineRun,
    Export,
}

/// Successful result for a single target.
#[derive(Debug, Clone)]
pub struct TargetSuccess {
    pub outcome: SyncOutcome,
}

/// Failure result for a single target.
#[derive(Debug)]
pub struct TargetFailure {
    pub raindex_id: RaindexIdentifier,
    pub raindex_key: Option<String>,
    pub stage: TargetStage,
    pub error: LocalDbError,
}

/// Aggregated result across all targets in a runner invocation.
#[derive(Debug, Default)]
pub struct RunReport {
    pub successes: Vec<TargetSuccess>,
    pub failures: Vec<TargetFailure>,
}

#[derive(Debug)]
pub enum RunOutcome {
    Report(RunReport),
    NotLeader,
}
