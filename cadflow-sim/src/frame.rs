//! Binary Frame Protocol encoding and decoding.
//!
//! Conforms to `frame_protocol.js`:
//! - Header: 12 bytes
//!   - `sim_time`: f64 (8 bytes, Little Endian)
//!   - `count`: u32 (4 bytes, Little Endian)
//! - Entity layout: 24 bytes per entity
//!   - `id`: u32 (4 bytes, Little Endian)
//!   - `entity_type`: u8 (1 byte)
//!   - `state`: u8 (1 byte)
//!   - `_pad`: [u8; 2] (2 bytes padding)
//!   - `x`: f32 (4 bytes, Little Endian)
//!   - `y`: f32 (4 bytes, Little Endian)
//!   - `z`: f32 (4 bytes, Little Endian)
//!   - `rot_y`: f32 (4 bytes, Little Endian)
//! - Optional Auxiliary Payload: UTF-8 JSON bytes appended after entity data.

use serde::{Deserialize, Serialize};

pub const HEADER_BYTES: usize = 12;
pub const ENTITY_BYTES: usize = 24;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntityFrameData {
    pub id: u32,
    pub entity_type: u8,
    pub state: u8,
    pub _pad: [u8; 2],
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rot_y: f32,
}

const _: () = assert!(std::mem::size_of::<EntityFrameData>() == 24);
const _: () = assert!(std::mem::align_of::<EntityFrameData>() == 4);

impl Default for EntityFrameData {
    fn default() -> Self {
        Self {
            id: 0,
            entity_type: 0,
            state: 0,
            _pad: [0, 0],
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rot_y: 0.0,
        }
    }
}

impl EntityFrameData {
    pub fn new(id: u32, entity_type: u8, state: u8, x: f32, y: f32, z: f32, rot_y: f32) -> Self {
        Self {
            id,
            entity_type,
            state,
            _pad: [0, 0],
            x,
            y,
            z,
            rot_y,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FrameEncoder {
    buffer: Vec<u8>,
}

impl FrameEncoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    /// Encode simulation frame into contiguous binary buffer.
    pub fn encode(
        &mut self,
        sim_time: f64,
        entities: &[EntityFrameData],
        auxiliary_json: Option<&str>,
    ) -> &[u8] {
        self.buffer.clear();
        let count = entities.len() as u32;
        let aux_len = auxiliary_json.map_or(0, |s| s.len());
        self.buffer.reserve(HEADER_BYTES + (count as usize) * ENTITY_BYTES + aux_len);

        // 1. Header (12 bytes)
        self.buffer.extend_from_slice(&sim_time.to_le_bytes());
        self.buffer.extend_from_slice(&count.to_le_bytes());

        // 2. Entity array
        if !entities.is_empty() {
            let slice = unsafe {
                std::slice::from_raw_parts(
                    entities.as_ptr() as *const u8,
                    entities.len() * ENTITY_BYTES,
                )
            };
            self.buffer.extend_from_slice(slice);
        }

        // 3. Auxiliary JSON bytes
        if let Some(json) = auxiliary_json {
            self.buffer.extend_from_slice(json.as_bytes());
        }

        &self.buffer
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

/// Decoded frame for Rust-side consumption.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedFrame {
    pub sim_time: f64,
    pub entities: Vec<EntityFrameData>,
    pub auxiliary_json: Option<String>,
}

/// Decode binary buffer matching frame protocol.
pub fn decode_frame(buf: &[u8]) -> Option<DecodedFrame> {
    if buf.len() < HEADER_BYTES {
        return None;
    }

    let sim_time = f64::from_le_bytes(buf[0..8].try_into().ok()?);
    let count = u32::from_le_bytes(buf[8..12].try_into().ok()?) as usize;

    let required_entity_bytes = count * ENTITY_BYTES;
    if buf.len() < HEADER_BYTES + required_entity_bytes {
        return None;
    }

    let mut entities = Vec::with_capacity(count);
    let mut offset = HEADER_BYTES;

    for _ in 0..count {
        let chunk = &buf[offset..offset + ENTITY_BYTES];
        let id = u32::from_le_bytes(chunk[0..4].try_into().ok()?);
        let entity_type = chunk[4];
        let state = chunk[5];
        let pad = [chunk[6], chunk[7]];
        let x = f32::from_le_bytes(chunk[8..12].try_into().ok()?);
        let y = f32::from_le_bytes(chunk[12..16].try_into().ok()?);
        let z = f32::from_le_bytes(chunk[16..20].try_into().ok()?);
        let rot_y = f32::from_le_bytes(chunk[20..24].try_into().ok()?);

        entities.push(EntityFrameData {
            id,
            entity_type,
            state,
            _pad: pad,
            x,
            y,
            z,
            rot_y,
        });
        offset += ENTITY_BYTES;
    }

    let auxiliary_json = if offset < buf.len() {
        std::str::from_utf8(&buf[offset..]).ok().map(|s| s.to_string())
    } else {
        None
    };

    Some(DecodedFrame {
        sim_time,
        entities,
        auxiliary_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_encode_decode() {
        let mut encoder = FrameEncoder::new();
        let entities = vec![
            EntityFrameData::new(1, 4, 1, 10.5, 0.0, -25.0, 1.57),
            EntityFrameData::new(2, 5, 0, 12.0, 1.2, -25.0, 0.0),
        ];
        let aux = r#"{"test":true}"#;

        let bytes = encoder.encode(12.345, &entities, Some(aux));
        assert_eq!(bytes.len(), HEADER_BYTES + 2 * ENTITY_BYTES + aux.len());

        let decoded = decode_frame(bytes).expect("Failed to decode frame");
        assert!((decoded.sim_time - 12.345).abs() < 1e-9);
        assert_eq!(decoded.entities.len(), 2);
        assert_eq!(decoded.entities[0].id, 1);
        assert_eq!(decoded.entities[0].entity_type, 4);
        assert!((decoded.entities[0].x - 10.5).abs() < 1e-6);
        assert_eq!(decoded.auxiliary_json.as_deref(), Some(aux));
    }
}
