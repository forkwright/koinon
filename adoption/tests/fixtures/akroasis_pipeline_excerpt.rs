// Excerpt captured verbatim from forkwright/akroasis
// crates/semaino/src/pipeline.rs. Every `koinon::` reference here resolves
// to akroasis's own local homonym crate (see akroasis_cargo_lock_koinon_entry.lock),
// not forkwright/koinon — none of GeoSignal / Timestamp match the real
// crate's top-level modules (telemetry, config, cli, error), so
// facts.find_references must return zero matches against this text.
//! Top-level async orchestrator wiring aggregation, convergence, and alerting.

use std::time::Duration;

use koinon::GeoSignal;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};

// ...

let evict_before_ms = koinon::Timestamp::now().as_unix_millis()
    - self.time_window.as_millis() as i64;
if let Ok(ts) = koinon::Timestamp::from_unix_millis(evict_before_ms) {
    self.grid.evict(ts);
}
