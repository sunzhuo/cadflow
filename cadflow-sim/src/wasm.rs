//! WebAssembly bindings via `wasm-bindgen`.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use crate::engine::DESEngine;
use crate::frame::EntityFrameData;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WasmDesEngine {
    inner: DESEngine,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WasmDesEngine {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(seed: f64) -> Self {
        let seed_u64 = if seed.is_finite() && seed >= 0.0 {
            seed as u64
        } else {
            12345
        };
        Self {
            inner: DESEngine::new(seed_u64),
        }
    }

    pub fn reset(&mut self, seed: f64, speed: f64, duration: f64) {
        let seed_u64 = if seed.is_finite() && seed >= 0.0 {
            seed as u64
        } else {
            12345
        };
        self.inner.reset(seed_u64, speed, duration);
    }

    pub fn next_id(&mut self) -> u32 {
        self.inner.next_id()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn clock(&self) -> f64 {
        self.inner.clock
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn speed(&self) -> f64 {
        self.inner.speed
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_speed(&mut self, speed: f64) {
        self.inner.speed = speed;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn duration(&self) -> f64 {
        self.inner.duration
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_duration(&mut self, duration: f64) {
        self.inner.duration = duration;
    }

    pub fn is_unlimited_speed(&self) -> bool {
        self.inner.is_unlimited_speed()
    }

    pub fn schedule_event(
        &mut self,
        delay: f64,
        event_type: String,
        data_json: Option<String>,
    ) -> bool {
        self.inner.schedule_event(delay, event_type, data_json)
    }

    pub fn peek_next_event_time(&self) -> f64 {
        self.inner.peek_next_event_time()
    }

    pub fn has_due_events(&self) -> bool {
        self.inner.has_due_events()
    }

    pub fn advance_to(&mut self, target_clock: f64) -> f64 {
        self.inner.advance_to(target_clock)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn dispatch_due_events(&mut self) -> js_sys::Array {
        let events = self.inner.dispatch_due_events();
        let arr = js_sys::Array::new_with_length(events.len() as u32);
        for (i, evt) in events.into_iter().enumerate() {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&obj, &"time".into(), &JsValue::from_f64(evt.time));
            let _ = js_sys::Reflect::set(&obj, &"type".into(), &JsValue::from_str(&evt.event_type));
            if let Some(json_str) = evt.data_json {
                if let Ok(parsed) = js_sys::JSON::parse(&json_str) {
                    let _ = js_sys::Reflect::set(&obj, &"data".into(), &parsed);
                } else {
                    let _ = js_sys::Reflect::set(&obj, &"data".into(), &JsValue::from_str(&json_str));
                }
            } else {
                let _ = js_sys::Reflect::set(&obj, &"data".into(), &JsValue::NULL);
            }
            arr.set(i as u32, obj.into());
        }
        arr
    }

    pub fn restore_to_clock(&mut self, target_clock: f64) -> f64 {
        self.inner.restore_to_clock(target_clock) as f64
    }

    // ── Entity Management ──

    pub fn spawn_entity(
        &mut self,
        id: u32,
        entity_type: u8,
        state: u8,
        x: f32,
        y: f32,
        z: f32,
        rot_y: f32,
    ) {
        let entity = EntityFrameData::new(id, entity_type, state, x, y, z, rot_y);
        self.inner.spawn_entity(entity);
    }

    pub fn remove_entity(&mut self, id: u32) -> bool {
        self.inner.remove_entity(id)
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
        self.inner.update_entity_pose(id, x, y, z, rot_y, state)
    }

    pub fn entity_count(&self) -> usize {
        self.inner.entity_count()
    }

    // ── Frame Encoding ──

    #[cfg(target_arch = "wasm32")]
    pub fn encode_current_frame(&mut self, auxiliary_json: Option<String>) -> js_sys::Uint8Array {
        let slice = self.inner.encode_current_frame(auxiliary_json.as_deref());
        // Return a copy as Uint8Array for postMessage transfer
        js_sys::Uint8Array::from(slice)
    }

    pub fn get_frame_buffer_ptr(&self) -> *const u8 {
        self.inner.frame_buffer().as_ptr()
    }

    pub fn get_frame_buffer_len(&self) -> usize {
        self.inner.frame_buffer().len()
    }

    // ── RNG ──

    pub fn rng_uniform(&mut self, low: f64, high: f64) -> f64 {
        self.inner.rng.uniform(low, high)
    }

    pub fn rng_unit(&mut self) -> f64 {
        self.inner.rng.unit()
    }

    pub fn rng_exponential(&mut self, lambda: f64) -> f64 {
        self.inner.rng.exponential(lambda)
    }

    pub fn rng_normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        self.inner.rng.normal(mean, std_dev)
    }

    pub fn rng_bool(&mut self, p: f64) -> bool {
        self.inner.rng.bool_with_prob(p)
    }

    // ── Message Bus ──

    pub fn bus_enqueue(&mut self, from: u32, to: u32, action: String, payload: Option<String>) {
        self.inner.bus.enqueue(from, to, action, payload);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn bus_flush(&mut self) -> js_sys::Array {
        let envelopes = self.inner.bus.flush();
        let arr = js_sys::Array::new_with_length(envelopes.len() as u32);
        for (i, env) in envelopes.into_iter().enumerate() {
            let obj = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&obj, &"from".into(), &JsValue::from_f64(env.from as f64));
            let _ = js_sys::Reflect::set(&obj, &"to".into(), &JsValue::from_f64(env.to as f64));
            let _ = js_sys::Reflect::set(&obj, &"action".into(), &JsValue::from_str(&env.action));
            if let Some(payload_str) = env.payload {
                let _ = js_sys::Reflect::set(&obj, &"payload".into(), &JsValue::from_str(&payload_str));
            } else {
                let _ = js_sys::Reflect::set(&obj, &"payload".into(), &JsValue::NULL);
            }
            arr.set(i as u32, obj.into());
        }
        arr
    }
}
