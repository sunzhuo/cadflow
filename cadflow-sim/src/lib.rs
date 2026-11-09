//! CADFlow Discrete Event Simulation (DES) Core Crate.

pub mod engine;
pub mod frame;
pub mod message_bus;
pub mod rng;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use engine::{DESEngine, SimEvent, CLOCK_EPSILON, DEFAULT_RUN_DURATION};
pub use frame::{EntityFrameData, FrameEncoder, DecodedFrame, decode_frame, HEADER_BYTES, ENTITY_BYTES};
pub use message_bus::{MessageBus, MessageEnvelope};
pub use rng::DeterministicRng;

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmDesEngine;
