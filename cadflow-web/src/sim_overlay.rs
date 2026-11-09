//! Simulation 2D Overlay Rendering and Web Worker Controller for CADFlow.
//!
//! Renders discrete-event simulation entities (IGV, cranes, containers, lock zones,
//! task markers) directly on the 2D CAD canvas using `egui::Painter`.

use egui::{pos2, Color32, Painter, Pos2, Stroke};
use cadflow_sim::{decode_frame, DecodedFrame, EntityFrameData};
use cadcraft_ui_egui::canvas::Xf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub const TYPE_LOCK_ZONE: u8 = 3;
pub const TYPE_IGV: u8 = 4;
pub const TYPE_CONTAINER: u8 = 5;
pub const TYPE_TASK_MARKER: u8 = 6;
pub const TYPE_YARD_CRANE: u8 = 7;

const IGV_LENGTH: f32 = 15.0;
const IGV_WIDTH: f32 = 2.75;
const CONTAINER_LENGTH: f32 = 12.192;
const CONTAINER_WIDTH: f32 = 2.591;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SIM_INBOX_FRAME: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
    static SIM_STATUS_TEXT: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static SIM_WORKER_REF: std::cell::RefCell<Option<web_sys::Worker>> = const { std::cell::RefCell::new(None) };
    static SIM_ONMESSAGE_HOLDER: std::cell::RefCell<Option<Closure<dyn FnMut(web_sys::MessageEvent)>>> = const { std::cell::RefCell::new(None) };
}

#[derive(Clone, Debug)]
pub struct SimController {
    pub is_running: bool,
    pub is_paused: bool,
    pub clock: f64,
    pub latest_frame: Option<DecodedFrame>,
    pub status_text: String,
    pub entity_count: usize,
    pub current_speed: f64,
}

impl Default for SimController {
    fn default() -> Self {
        Self {
            is_running: false,
            is_paused: false,
            clock: 0.0,
            latest_frame: None,
            status_text: "离散事件仿真就绪".into(),
            entity_count: 0,
            current_speed: 1.0,
        }
    }
}

impl SimController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Poll incoming frame and status messages from worker.
    pub fn poll_messages(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            SIM_INBOX_FRAME.with(|cell| {
                if let Some(bytes) = cell.borrow_mut().take() {
                    self.update_frame_bytes(&bytes);
                }
            });
            SIM_STATUS_TEXT.with(|cell| {
                if let Some(txt) = cell.borrow_mut().take() {
                    self.status_text = txt;
                }
            });
        }
    }

    /// Start simulation with given scene script content or default module.
    pub fn start(&mut self, scene_script: Option<&str>, speed: f64, seed: u64) {
        self.is_running = true;
        self.is_paused = false;
        self.current_speed = speed;
        self.status_text = "正在启动仿真引擎...".into();

        #[cfg(target_arch = "wasm32")]
        {
            self.ensure_worker();

            let msg = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&msg, &"type".into(), &"start".into());
            let _ = js_sys::Reflect::set(&msg, &"speed".into(), &JsValue::from_f64(speed));
            let _ = js_sys::Reflect::set(&msg, &"seed".into(), &JsValue::from_f64(seed as f64));

            if let Some(script) = scene_script {
                let _ = js_sys::Reflect::set(&msg, &"sceneScript".into(), &JsValue::from_str(script));
            } else {
                // Default to built-in nb1 scenario
                let _ = js_sys::Reflect::set(&msg, &"sceneModuleUrl".into(), &JsValue::from_str("./assets/sim/apps/nb1/code/scene.js"));
            }

            SIM_WORKER_REF.with(|cell| {
                if let Some(worker) = cell.borrow().as_ref() {
                    let _ = worker.post_message(&msg);
                }
            });
        }
    }

    pub fn pause(&mut self) {
        if !self.is_running || self.is_paused {
            return;
        }
        self.is_paused = true;
        self.status_text = "仿真已暂停".into();

        #[cfg(target_arch = "wasm32")]
        {
            let msg = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&msg, &"type".into(), &"pause".into());
            SIM_WORKER_REF.with(|cell| {
                if let Some(worker) = cell.borrow().as_ref() {
                    let _ = worker.post_message(&msg);
                }
            });
        }
    }

    pub fn resume(&mut self) {
        if !self.is_running || !self.is_paused {
            return;
        }
        self.is_paused = false;
        self.status_text = "仿真运行中".into();

        #[cfg(target_arch = "wasm32")]
        {
            let msg = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&msg, &"type".into(), &"resume".into());
            SIM_WORKER_REF.with(|cell| {
                if let Some(worker) = cell.borrow().as_ref() {
                    let _ = worker.post_message(&msg);
                }
            });
        }
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        self.is_paused = false;
        self.status_text = "仿真已停止".into();

        #[cfg(target_arch = "wasm32")]
        {
            let msg = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&msg, &"type".into(), &"stop".into());
            SIM_WORKER_REF.with(|cell| {
                if let Some(worker) = cell.borrow().as_ref() {
                    let _ = worker.post_message(&msg);
                }
            });
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.current_speed = speed;
        #[cfg(target_arch = "wasm32")]
        {
            let msg = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&msg, &"type".into(), &"setSpeed".into());
            let _ = js_sys::Reflect::set(&msg, &"speed".into(), &JsValue::from_f64(speed));
            SIM_WORKER_REF.with(|cell| {
                if let Some(worker) = cell.borrow().as_ref() {
                    let _ = worker.post_message(&msg);
                }
            });
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn ensure_worker(&mut self) {
        let has_worker = SIM_WORKER_REF.with(|cell| cell.borrow().is_some());
        if has_worker {
            return;
        }

        let options = web_sys::WorkerOptions::new();
        options.set_type(web_sys::WorkerType::Module);
        let worker_res = web_sys::Worker::new_with_options("./assets/sim/des_worker.js", &options);
        match worker_res {
            Ok(worker) => {
                let onerror = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
                    let err_msg = format!("DES Worker 运行错误: {}", e.message());
                    web_sys::console::error_1(&err_msg.clone().into());
                    SIM_STATUS_TEXT.with(|st| *st.borrow_mut() = Some(err_msg));
                }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
                worker.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                onerror.forget();

                let onmessage = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                    let data = e.data();
                    if let Ok(buf) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
                        let u8_arr = js_sys::Uint8Array::new(&buf);
                        let mut vec = vec![0u8; u8_arr.length() as usize];
                        u8_arr.copy_to(&mut vec);
                        SIM_INBOX_FRAME.with(|f| *f.borrow_mut() = Some(vec));
                    } else if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
                        if let Ok(typ) = js_sys::Reflect::get(&obj, &"type".into()) {
                            if typ == "status" {
                                if let Ok(txt) = js_sys::Reflect::get(&obj, &"text".into()) {
                                    if let Some(s) = txt.as_string() {
                                        SIM_STATUS_TEXT.with(|st| *st.borrow_mut() = Some(s));
                                    }
                                }
                            } else if typ == "error" {
                                if let Ok(txt) = js_sys::Reflect::get(&obj, &"text".into()) {
                                    if let Some(s) = txt.as_string() {
                                        let err = format!("仿真错误: {s}");
                                        web_sys::console::error_1(&err.clone().into());
                                        SIM_STATUS_TEXT.with(|st| *st.borrow_mut() = Some(err));
                                    }
                                }
                            } else if typ == "frame" {
                                if let Ok(buf_val) = js_sys::Reflect::get(&obj, &"buffer".into()) {
                                    if let Ok(buf) = buf_val.dyn_into::<js_sys::ArrayBuffer>() {
                                        let u8_arr = js_sys::Uint8Array::new(&buf);
                                        let mut vec = vec![0u8; u8_arr.length() as usize];
                                        u8_arr.copy_to(&mut vec);
                                        SIM_INBOX_FRAME.with(|f| *f.borrow_mut() = Some(vec));
                                    }
                                }
                            }
                        }
                    }
                }) as Box<dyn FnMut(web_sys::MessageEvent)>);

                worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

                SIM_ONMESSAGE_HOLDER.with(|cell| *cell.borrow_mut() = Some(onmessage));
                SIM_WORKER_REF.with(|cell| *cell.borrow_mut() = Some(worker));
            }
            Err(e) => {
                let err = format!("创建仿真 Worker 失败: {:?}", e);
                web_sys::console::error_1(&err.clone().into());
                self.status_text = err;
            }
        }
    }

    /// Update received binary frame from worker or message queue.
    pub fn update_frame_bytes(&mut self, bytes: &[u8]) {
        if let Some(frame) = decode_frame(bytes) {
            self.clock = frame.sim_time;
            self.entity_count = frame.entities.len();
            self.latest_frame = Some(frame);
        }
    }

    /// Render simulation entities over CAD canvas.
    pub fn render(&self, painter: &Painter, xf: &Xf) {
        let Some(frame) = &self.latest_frame else {
            return;
        };

        // 1. Draw Lock Zones first (background translucent layer)
        for entity in &frame.entities {
            if entity.entity_type == TYPE_LOCK_ZONE {
                self.draw_lock_zone(painter, xf, entity);
            }
        }

        // 2. Draw Yard Cranes
        for entity in &frame.entities {
            if entity.entity_type == TYPE_YARD_CRANE {
                self.draw_crane(painter, xf, entity);
            }
        }

        // 3. Draw IGVs (Automated Guided Vehicles)
        for entity in &frame.entities {
            if entity.entity_type == TYPE_IGV {
                self.draw_igv(painter, xf, entity);
            }
        }

        // 4. Draw Containers
        for entity in &frame.entities {
            if entity.entity_type == TYPE_CONTAINER {
                self.draw_container(painter, xf, entity);
            }
        }

        // 5. Draw Task Markers
        for entity in &frame.entities {
            if entity.entity_type == TYPE_TASK_MARKER {
                self.draw_marker(painter, xf, entity);
            }
        }
    }

    /// Transform simulation coordinate (x, z) to egui screen Pos2.
    /// In cad2sim: CAD X = sim.x, CAD Y = -sim.z.
    fn to_screen_pos(xf: &Xf, x: f32, z: f32) -> Pos2 {
        let cad_pos = cadcraft_geom::Vec2::new(x as f64, -z as f64);
        xf.to_screen(cad_pos)
    }

    fn draw_igv(&self, painter: &Painter, xf: &Xf, entity: &EntityFrameData) {
        let center = Self::to_screen_pos(xf, entity.x, entity.z);
        let scale = xf.scale as f32;

        let len_px = (IGV_LENGTH * scale).max(12.0);
        let width_px = (IGV_WIDTH * scale).max(5.0);

        let rot = -entity.rot_y;
        let cos_r = rot.cos();
        let sin_r = rot.sin();

        let half_l = len_px / 2.0;
        let half_w = width_px / 2.0;

        let p1 = pos2(center.x + (-half_l * cos_r - -half_w * sin_r), center.y + (-half_l * sin_r + -half_w * cos_r));
        let p2 = pos2(center.x + (half_l * cos_r - -half_w * sin_r), center.y + (half_l * sin_r + -half_w * cos_r));
        let p3 = pos2(center.x + (half_l * cos_r - half_w * sin_r), center.y + (half_l * sin_r + half_w * cos_r));
        let p4 = pos2(center.x + (-half_l * cos_r - half_w * sin_r), center.y + (-half_l * sin_r + half_w * cos_r));

        let body_color = match entity.state {
            0 => Color32::from_rgb(102, 187, 106),
            1 => Color32::from_rgb(66, 165, 245),
            2 => Color32::from_rgb(255, 167, 38),
            _ => Color32::from_rgb(171, 71, 188),
        };

        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3, p4],
            body_color,
            Stroke::new(1.0, Color32::WHITE),
        ));

        if len_px > 16.0 {
            let tip = pos2(center.x + (half_l * 0.9 * cos_r), center.y + (half_l * 0.9 * sin_r));
            painter.line_segment([center, tip], Stroke::new(2.0, Color32::YELLOW));
        }
    }

    fn draw_container(&self, painter: &Painter, xf: &Xf, entity: &EntityFrameData) {
        let center = Self::to_screen_pos(xf, entity.x, entity.z);
        let scale = xf.scale as f32;

        let len_px = (CONTAINER_LENGTH * scale).max(10.0);
        let width_px = (CONTAINER_WIDTH * scale).max(4.0);

        let rot = -entity.rot_y;
        let cos_r = rot.cos();
        let sin_r = rot.sin();

        let half_l = len_px / 2.0;
        let half_w = width_px / 2.0;

        let p1 = pos2(center.x + (-half_l * cos_r - -half_w * sin_r), center.y + (-half_l * sin_r + -half_w * cos_r));
        let p2 = pos2(center.x + (half_l * cos_r - -half_w * sin_r), center.y + (half_l * sin_r + -half_w * cos_r));
        let p3 = pos2(center.x + (half_l * cos_r - half_w * sin_r), center.y + (half_l * sin_r + half_w * cos_r));
        let p4 = pos2(center.x + (-half_l * cos_r - half_w * sin_r), center.y + (-half_l * sin_r + half_w * cos_r));

        let box_color = match entity.state % 6 {
            0 => Color32::from_rgb(207, 107, 82),
            1 => Color32::from_rgb(212, 162, 76),
            2 => Color32::from_rgb(93, 160, 209),
            3 => Color32::from_rgb(114, 181, 109),
            4 => Color32::from_rgb(163, 123, 199),
            _ => Color32::from_rgb(201, 139, 153),
        };

        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3, p4],
            box_color,
            Stroke::new(1.0, Color32::from_black_alpha(180)),
        ));
    }

    fn draw_crane(&self, painter: &Painter, xf: &Xf, entity: &EntityFrameData) {
        let center = Self::to_screen_pos(xf, entity.x, entity.z);
        let scale = xf.scale as f32;

        let span_px = (30.0 * scale).max(20.0);
        let rail_len_px = (12.0 * scale).max(8.0);

        let left_rail = pos2(center.x - span_px / 2.0, center.y);
        let right_rail = pos2(center.x + span_px / 2.0, center.y);

        let crane_color = Color32::from_rgb(230, 162, 60);
        painter.line_segment([left_rail, right_rail], Stroke::new(3.0, crane_color));

        painter.line_segment(
            [pos2(left_rail.x, left_rail.y - rail_len_px / 2.0), pos2(left_rail.x, left_rail.y + rail_len_px / 2.0)],
            Stroke::new(2.5, Color32::WHITE),
        );
        painter.line_segment(
            [pos2(right_rail.x, right_rail.y - rail_len_px / 2.0), pos2(right_rail.x, right_rail.y + rail_len_px / 2.0)],
            Stroke::new(2.5, Color32::WHITE),
        );

        painter.circle_filled(center, 4.0, Color32::from_rgb(255, 87, 34));
    }

    fn draw_lock_zone(&self, painter: &Painter, xf: &Xf, entity: &EntityFrameData) {
        let center = Self::to_screen_pos(xf, entity.x, entity.z);
        let r = (8.0 * xf.scale as f32).max(4.0);
        let fill = match entity.state {
            0 => Color32::from_rgba_unmultiplied(102, 187, 106, 60),
            _ => Color32::from_rgba_unmultiplied(239, 83, 80, 80),
        };
        painter.circle_filled(center, r, fill);
    }

    fn draw_marker(&self, painter: &Painter, xf: &Xf, entity: &EntityFrameData) {
        let center = Self::to_screen_pos(xf, entity.x, entity.z);
        let marker_color = if entity.state == 0 {
            Color32::from_rgb(77, 151, 255)
        } else {
            Color32::from_rgb(232, 93, 93)
        };
        painter.circle_filled(center, 5.0, marker_color);
        painter.circle_stroke(center, 7.0, Stroke::new(1.5, Color32::WHITE));
    }
}
