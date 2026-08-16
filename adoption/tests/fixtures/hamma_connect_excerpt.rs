// Excerpt captured verbatim from forkwright/hamma
// crates/dictyon/examples/connect.rs (lines 1-20, 92-95) — the file issue
// #19 itself cites as hamma's adoption evidence. Line numbers in this
// excerpt intentionally do NOT match the real file; test_derive.py's
// reference test asserts on content (`module == "telemetry"`), not on a
// specific line number, for exactly that reason.
use dictyon::control::{ControlClient, ControlError, RegisterOutcome};
use dictyon::noise::{NoiseError, NoiseHandshake};
use dictyon::transport::{ControlConnection, TransportError};
use dictyon::wire::{AsyncControlStream, ControlConfig, WireError, connect};
use koinon::telemetry;
use mitos::keys::{DiscoPrivate, MachinePrivate, NodePrivate};
use snafu::{ResultExt, Snafu};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), ExampleError> {
    telemetry::init("dictyon=debug");
    run().await
}
