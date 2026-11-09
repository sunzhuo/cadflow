//! Core Discrete Event Simulation (DES) Engine.
//!
//! Features:
//! - Priority queue for event dispatching with sub-microsecond precision
//! - Deterministic PRNG with seed-based reproducibility
//! - Zero-copy binary frame generation
//! - Headless fast-forwarding (`restore_to_clock`) in closed-loop native code
//! - Inter-agent message bus

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use crate::frame::{EntityFrameData, FrameEncoder};
use crate::message_bus::MessageBus;
use crate::rng::DeterministicRng;

pub const CLOCK_EPSILON: f64 = 1e-9;
pub const DEFAULT_RUN_DURATION: f64 = f64::INFINITY;

/// A discrete simulation event.
#[derive(Clone, Debug, PartialEq)]
pub struct SimEvent {
    pub time: f64,
    pub event_id: u64,
    pub event_type: String,
    pub data_json: Option<String>,
}

impl Eq for SimEvent {}

impl Ord for SimEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: smaller time has higher priority; break ties with event_id FIFO order
        other
            .time
            .partial_cmp(&self.time)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.event_id.cmp(&self.event_id))
    }
}

impl PartialOrd for SimEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Statistics for simulation run.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EngineStats {
    pub events_processed: u64,
    pub frames_published: u64,
}

pub struct DESEngine {
    pub clock: f64,
    pub speed: f64,
    pub duration: f64,
    queue: BinaryHeap<SimEvent>,
    next_event_id: u64,
    next_agent_id: u32,
    pub rng: DeterministicRng,
    entities: Vec<EntityFrameData>,
    entity_index_map: HashMap<u32, usize>,
    pub bus: MessageBus,
    frame_encoder: FrameEncoder,
    pub stats: EngineStats,
    restoring: bool,
}

impl DESEngine {
    pub fn new(seed: u64) -> Self {
        Self {
            clock: 0.0,
            speed: 1.0,
            duration: DEFAULT_RUN_DURATION,
            queue: BinaryHeap::with_capacity(1024),
            next_event_id: 1,
            next_agent_id: 1,
            rng: DeterministicRng::new(seed),
            entities: Vec::with_capacity(256),
            entity_index_map: HashMap::with_capacity(256),
            bus: MessageBus::new(),
            frame_encoder: FrameEncoder::new(),
            stats: EngineStats::default(),
            restoring: false,
        }
    }

    /// Reset engine state for a new simulation run.
    pub fn reset(&mut self, seed: u64, speed: f64, duration: f64) {
        self.clock = 0.0;
        self.speed = if speed > 0.0 { speed } else { 1.0 };
        self.duration = if duration > 0.0 { duration } else { DEFAULT_RUN_DURATION };
        self.queue.clear();
        self.next_event_id = 1;
        self.next_agent_id = 1;
        self.rng.reseed(seed);
        self.entities.clear();
        self.entity_index_map.clear();
        self.bus.clear();
        self.stats = EngineStats::default();
        self.restoring = false;
    }

    /// Allocate next incremental unique agent ID.
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_agent_id;
        self.next_agent_id += 1;
        id
    }

    pub fn is_unlimited_speed(&self) -> bool {
        self.speed.is_infinite() || self.speed < 0.0
    }

    /// Bound target clock by duration limit.
    pub fn bound_target_clock(&self, target_clock: f64) -> f64 {
        if self.duration.is_finite() {
            target_clock.min(self.duration)
        } else {
            target_clock
        }
    }

    /// Schedule a new event at `clock + delay`.
    pub fn schedule_event(
        &mut self,
        delay: f64,
        event_type: impl Into<String>,
        data_json: Option<String>,
    ) -> bool {
        if !delay.is_finite() || delay < 0.0 {
            return false;
        }

        let event_time = self.clock + delay;
        let event_id = self.next_event_id;
        self.next_event_id += 1;

        self.queue.push(SimEvent {
            time: event_time,
            event_id,
            event_type: event_type.into(),
            data_json,
        });

        true
    }

    /// Peek the timestamp of the next event in the queue.
    pub fn peek_next_event_time(&self) -> f64 {
        self.queue.peek().map_or(f64::INFINITY, |e| e.time)
    }

    /// Returns true if there is an event due at or before the current clock + epsilon.
    pub fn has_due_events(&self) -> bool {
        match self.queue.peek() {
            Some(evt) => evt.time <= self.clock + CLOCK_EPSILON,
            None => false,
        }
    }

    /// Advance clock up to bounded `target_clock`.
    pub fn advance_to(&mut self, target_clock: f64) -> f64 {
        let bounded = self.bound_target_clock(target_clock);
        if bounded + CLOCK_EPSILON < self.clock {
            return self.clock;
        }

        self.clock = bounded;
        let _ = self.bus.flush();
        self.clock
    }

    /// Pop and return all events due at the current clock.
    pub fn dispatch_due_events(&mut self) -> Vec<SimEvent> {
        let mut dispatched = Vec::new();

        while let Some(evt) = self.queue.peek() {
            if evt.time <= self.clock + CLOCK_EPSILON {
                let evt = self.queue.pop().unwrap();
                self.clock = self.clock.max(evt.time);
                self.stats.events_processed += 1;
                dispatched.push(evt);
            } else {
                break;
            }
        }

        let _ = self.bus.flush();
        dispatched
    }

    /// Fast-forward simulation headlessly to `target_clock` in pure native code.
    pub fn restore_to_clock(&mut self, target_clock: f64) -> u64 {
        let bounded = self.bound_target_clock(target_clock);
        if !(bounded > self.clock + CLOCK_EPSILON) {
            return 0;
        }

        self.restoring = true;
        let mut processed = 0;

        while self.clock + CLOCK_EPSILON < bounded {
            let next_event_time = self.peek_next_event_time();
            let next_clock = if next_event_time.is_finite() {
                bounded.min(next_event_time)
            } else {
                bounded
            };

            if next_clock > self.clock + CLOCK_EPSILON {
                self.advance_to(next_clock);
            }

            while self.has_due_events() {
                if let Some(evt) = self.queue.pop() {
                    self.clock = self.clock.max(evt.time);
                    self.stats.events_processed += 1;
                    processed += 1;
                }
            }

            if self.duration.is_finite() && self.clock >= self.duration - CLOCK_EPSILON {
                break;
            }

            if !next_event_time.is_finite() && next_clock >= bounded - CLOCK_EPSILON {
                break;
            }
        }

        self.restoring = false;
        processed
    }

    // ── Entity Management ──

    pub fn spawn_entity(&mut self, entity: EntityFrameData) {
        if let Some(&idx) = self.entity_index_map.get(&entity.id) {
            self.entities[idx] = entity;
        } else {
            let idx = self.entities.len();
            self.entity_index_map.insert(entity.id, idx);
            self.entities.push(entity);
        }
    }

    pub fn remove_entity(&mut self, id: u32) -> bool {
        if let Some(idx) = self.entity_index_map.remove(&id) {
            self.entities.swap_remove(idx);
            // Fix swapped element index
            if idx < self.entities.len() {
                let swapped_id = self.entities[idx].id;
                self.entity_index_map.insert(swapped_id, idx);
            }
            true
        } else {
            false
        }
    }

    pub fn update_entity_pose(
        &mut self,
        id: u32,
        x: f32,
        y: f32,
        z: f32,
        rot_y: f32,
        state: u8,
    ) -> bool {
        if let Some(&idx) = self.entity_index_map.get(&id) {
            let e = &mut self.entities[idx];
            e.x = x;
            e.y = y;
            e.z = z;
            e.rot_y = rot_y;
            e.state = state;
            true
        } else {
            false
        }
    }

    pub fn get_entity(&self, id: u32) -> Option<&EntityFrameData> {
        self.entity_index_map.get(&id).map(|&idx| &self.entities[idx])
    }

    pub fn entities(&self) -> &[EntityFrameData] {
        &self.entities
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    // ── Frame Encoding ──

    pub fn encode_current_frame(&mut self, auxiliary_json: Option<&str>) -> &[u8] {
        self.stats.frames_published += 1;
        self.frame_encoder
            .encode(self.clock, &self.entities, auxiliary_json)
    }

    pub fn frame_buffer(&self) -> &[u8] {
        self.frame_encoder.buffer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_event_queue() {
        let mut engine = DESEngine::new(42);
        engine.schedule_event(5.0, "evt_five", None);
        engine.schedule_event(2.0, "evt_two", None);
        engine.schedule_event(10.0, "evt_ten", None);

        assert!((engine.peek_next_event_time() - 2.0).abs() < 1e-9);

        // Advance to 3.0
        engine.advance_to(3.0);
        assert!((engine.clock - 3.0).abs() < 1e-9);
        assert!(engine.has_due_events());

        let due = engine.dispatch_due_events();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].event_type, "evt_two");

        // Advance to 10.0
        engine.advance_to(10.0);
        let due = engine.dispatch_due_events();
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].event_type, "evt_five");
        assert_eq!(due[1].event_type, "evt_ten");
    }

    #[test]
    fn test_entity_management() {
        let mut engine = DESEngine::new(42);
        let e1 = EntityFrameData::new(101, 4, 0, 1.0, 2.0, 3.0, 0.5);
        let e2 = EntityFrameData::new(102, 5, 1, 4.0, 5.0, 6.0, 1.0);

        engine.spawn_entity(e1);
        engine.spawn_entity(e2);
        assert_eq!(engine.entity_count(), 2);

        engine.update_entity_pose(101, 10.0, 20.0, 30.0, 1.5, 2);
        let updated = engine.get_entity(101).expect("entity 101 missing");
        assert!((updated.x - 10.0).abs() < 1e-6);
        assert_eq!(updated.state, 2);

        assert!(engine.remove_entity(101));
        assert_eq!(engine.entity_count(), 1);
        assert!(engine.get_entity(101).is_none());
        assert!(engine.get_entity(102).is_some());
    }

    #[test]
    fn test_headless_restore() {
        let mut engine = DESEngine::new(42);
        for i in 1..=20 {
            engine.schedule_event(i as f64 * 0.5, "tick", None);
        }

        let processed = engine.restore_to_clock(5.0);
        assert_eq!(processed, 10);
        assert!((engine.clock - 5.0).abs() < 1e-9);
        assert_eq!(engine.stats.events_processed, 10);
    }
}
