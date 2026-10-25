//! CADFlow in the browser.
//!
//! Web CAD with full Chinese interface and unrestricted Markdown documentation panel.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_arch = "wasm32")]
pub mod download;
#[cfg(target_arch = "wasm32")]
pub mod buerligons;
#[cfg(target_arch = "wasm32")]
pub mod doc_panel;
#[cfg(target_arch = "wasm32")]
pub mod sidebar;
#[cfg(target_arch = "wasm32")]
mod host;
#[cfg(target_arch = "wasm32")]
mod upload;

#[cfg(target_arch = "wasm32")]
mod web {
    use std::sync::Arc;
    use cadcraft_engine::Session;
    use cadcraft_ui_egui::docking::Panel;
    use cadcraft_ui_egui::theme::Tokens;
    use cadcraft_ui_egui::{CadApp, Services};
    use craft_ui::docking::{Layout, Node};
    use egui::{Color32, Stroke};
    use wasm_bindgen::JsCast as _;

    const CANVAS_ID: &str = "cadcraft_canvas";
    const LOADING_ID: &str = "cadcraft_loading";

    /// CADFlow Web Shell: integrates CADCraft engine & UI with Markdown documentation panel.
    struct Shell {
        app: CadApp,
        inbox: crate::upload::Inbox,
        host: Option<(crate::host::Bridge, crate::host::DirtyReport)>,
        reduced_motion: Option<web_sys::MediaQueryList>,
        doc_state: crate::doc_panel::DocState,
        show_doc_panel: bool,
        doc_width: f32,
        show_sidebar: bool,
        sidebar_width: f32,
        sidebar_state: crate::sidebar::SidebarState,
        font_data: Option<Arc<egui::FontData>>,
        font_installed: bool,
    }

    impl eframe::App for Shell {
        fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.inbox.take_dropped(ctx);
            self.inbox.drain(&mut self.app);
            self.app.system_animations = self.reduced_motion.as_ref().map(|reduced| !reduced.matches());

            // 保持无标签页并直显当前绘图
            self.app.ui.show_file_tabs = false;
            self.app.ui.start_tab = false;
            if self.app.session.docs.is_empty() {
                let _ = self.app.run("new", serde_json::json!({}));
            }

            self.app.logic(ctx);

            // 在 CadApp::logic 完成后注册中文字体（避免被其初次 install_fonts 覆盖）
            if !self.font_installed
                && let Some(font_data) = &self.font_data
            {
                let mut fonts = egui::FontDefinitions::default();
                fonts.font_data.insert("chinese-custom".into(), font_data.clone());
                if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    f.insert(0, "chinese-custom".into());
                }
                if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    f.push("chinese-custom".into());
                }
                ctx.set_fonts(fonts);
                self.font_installed = true;
            }

            if let Some((bridge, reported)) = &self.host {
                bridge.report_dirty(&self.app, reported);
            }
        }

        fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
            self.app.raw_input_hook(raw);
            cadcraft_ui_egui::cmdline::capture_tab(ctx, raw);
        }

        fn save(&mut self, storage: &mut dyn eframe::Storage) {
            storage.set_string(cadcraft_ui_egui::options::PROFILE_KEY, self.app.profile_json());
            storage.set_string(cadcraft_ui_egui::PREFS_KEY, self.app.prefs_json());
        }

        fn persist_egui_memory(&self) -> bool {
            false
        }

        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            // 隐藏 cadcraft 原生工具栏、独立横向菜单栏和标签页
            self.app.ui.show_toolbar = false;
            self.app.ui.in_window_menu = false;
            self.app.ui.show_file_tabs = false;
            self.app.ui.start_tab = false;

            // 1. 顶部渲染 Buerligons 风格工具栏 (固定 48px 高)
            crate::buerligons::draw_top_toolbar(
                ui,
                &mut self.app,
                &mut self.show_sidebar,
                &mut self.sidebar_width,
                &mut self.show_doc_panel,
                &mut self.doc_width,
                &mut self.doc_state,
            );

            // 2. 左侧侧边栏面板（包含工具集/图层/块/组与特性）
            if self.show_sidebar {
                let sb_resp = egui::Panel::left("cadflow_left_sidebar")
                    .resizable(true)
                    .default_size(self.sidebar_width)
                    .min_size(crate::buerligons::SIDEBAR_MIN_WIDTH)
                    .max_size(crate::buerligons::SIDEBAR_MAX_WIDTH)
                    .frame(
                        egui::Frame::NONE
                            .fill(Tokens::get().panel)
                            .stroke(Stroke::new(1.0, Tokens::get().border)),
                    )
                    .show(ui, |ui| {
                        self.sidebar_state.ui(ui, &mut self.app);
                    });
                self.sidebar_width = sb_resp.response.rect.width();
            }

            // 3. 右侧文档面板（平滑滑动展开/折叠，纯白底色，StackEdit 样式）
            let slide_progress = ui.ctx().animate_bool(egui::Id::new("cadflow_doc_slide"), self.show_doc_panel);
            if slide_progress > 0.001 {
                let full_w = self.doc_width.clamp(crate::buerligons::DOC_MIN_WIDTH, crate::buerligons::DOC_MAX_WIDTH);
                let effective_w = full_w * slide_progress;

                let doc_resp = egui::Panel::right("cadflow_document_side_panel")
                    .resizable(slide_progress >= 0.99)
                    .exact_size(effective_w)
                    .min_size(crate::buerligons::DOC_MIN_WIDTH)
                    .max_size(crate::buerligons::DOC_MAX_WIDTH)
                    .frame(
                        egui::Frame::NONE
                            .fill(Color32::WHITE)
                            .stroke(Stroke::new(1.0, Tokens::get().border)),
                    )
                    .show(ui, |ui| {
                        self.doc_state.ui(ui);
                    });

                if slide_progress >= 0.99 {
                    self.doc_width = doc_resp.response.rect.width();
                }
            }

            // 4. CAD 核心绘图画布（占据其余空间）
            self.app.ui(ui);

            if let Some((bridge, reported)) = &self.host {
                bridge.report_dirty(&self.app, reported);
            }
        }
    }

    fn query() -> String {
        web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default()
    }

    fn query_param(key: &str) -> Option<String> {
        web_sys::UrlSearchParams::new_with_str(&query()).ok()?.get(key)
    }

    fn services() -> Services {
        Services {
            pick_save: Some(Box::new(|name: &str| Some(name.to_string()))),
            ..Services::default()
        }
    }

    async fn fetch_font(path: &str) -> Option<Vec<u8>> {
        let window = web_sys::window()?;
        let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(path)).await.ok()?;
        let resp: web_sys::Response = resp_value.dyn_into().ok()?;
        if !resp.ok() {
            return None;
        }
        let array_buf_val = wasm_bindgen_futures::JsFuture::from(resp.array_buffer().ok()?).await.ok()?;
        let array = js_sys::Uint8Array::new(&array_buf_val);
        Some(array.to_vec())
    }

    /// 构建初始停靠布局：主中央绘图区 (Canvas) 占满空间，其他面板隐藏至外层独立侧边栏
    fn initial_docking_workspace() -> cadcraft_ui_egui::docking::Workspace {
        cadcraft_ui_egui::docking::Workspace {
            layout: Layout {
                root: Some(Node::Tabs {
                    panels: vec![Panel::Canvas],
                    active: 0,
                }),
                floating: Vec::new(),
            },
            hidden: Vec::new(),
        }
    }

    pub fn start() {
        eframe::WebLogger::init(log::LevelFilter::Info).ok();
        cadcraft_engine::cmd::file::set_io(cadcraft_engine::cmd::file::IoHooks {
            read: |b, name| cadcraft_io::read(b, name).map_err(|e| e.to_string()),
            write: |d, name| cadcraft_io::write(d, name).map_err(|e| e.to_string()),
            plot: Some(|d, space, opts| cadcraft_io::plotter::plot(d, space, opts).map_err(|e| e.to_string())),
        });
        cadcraft_engine::cmd::file::set_write_as(|d, name, o| {
            cadcraft_io::write_with(d, name, &cadcraft_io::SaveOptions { version: o.version, binary: o.binary }).map_err(|e| e.to_string())
        });
        cadcraft_engine::cmd::file::set_entity_codec(cadcraft_engine::cmd::file::EntityCodec {
            groups: cadcraft_io::entity_groups,
            entity: cadcraft_io::entity_from_groups,
        });

        let bridge = if query_param("host").as_deref() == Some("parent") { crate::host::Bridge::new() } else { None };
        match &bridge {
            Some(bridge) => {
                bridge.install();
                cadcraft_engine::cmd::file::set_deliver(crate::host::deliver);
            }
            None => cadcraft_engine::cmd::file::set_deliver(crate::download::download),
        }
        let app_bridge = bridge.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
            let Some(canvas) = document.get_element_by_id(CANVAS_ID).and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()) else {
                log::error!("missing <canvas id=\"{CANVAS_ID}\">");
                return;
            };

            // 预加载中文字体
            let font_bytes = fetch_font("assets/font.ttf").await;
            let font_data = font_bytes.map(|b| Arc::new(egui::FontData::from_owned(b)));

            let mut options = eframe::WebOptions::default();
            if query().contains("webgl")
                && let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup
            {
                create.instance_descriptor.backends = eframe::wgpu::Backends::GL;
            }

            let result = eframe::WebRunner::new()
                .start(
                    canvas,
                    options,
                    Box::new(move |cc| {
                        let session = if app_bridge.is_some() { Session::empty() } else { Session::new() };
                        let mut app = CadApp::new(session, services());

                        // 隐藏原生工具栏、横向菜单栏和标签页
                        app.ui.show_toolbar = false;
                        app.ui.in_window_menu = false;
                        app.ui.show_file_tabs = false;
                        app.ui.start_tab = false;

                        // 设置中文界面及系统语言首选项
                        app.ui.interface_language = cadcraft_ui_egui::i18n::Preference::Chinese;
                        app.system_languages = vec!["zh-CN".into(), "zh".into()];

                        if let Some(profile) = cc.storage.and_then(|s| s.get_string(cadcraft_ui_egui::options::PROFILE_KEY)) {
                            app.load_profile(&profile);
                        }
                        if let Some(prefs) = cc.storage.and_then(|s| s.get_string(cadcraft_ui_egui::PREFS_KEY)) {
                            app.load_prefs(&prefs);
                        }

                        app.ui.interface_language = cadcraft_ui_egui::i18n::Preference::Chinese;
                        app.ui.docking = initial_docking_workspace();

                        // 确保初始有一个活动空白图纸，直接显示绘图区 (Req 6)
                        if app.session.docs.is_empty() {
                            let _ = app.run("new", serde_json::json!({}));
                        }

                        if !crate::wants_cpu(&query())
                            && let Some(rs) = &cc.wgpu_render_state
                        {
                            app.set_wgpu(rs, None);
                        }
                        if query().contains("sample") {
                            let _ = app.run("ui.sample", serde_json::json!({}));
                        }

                        let inbox = crate::upload::Inbox::new(&cc.egui_ctx);
                        let picker = inbox.clone();
                        app.services.pick_open = Some(Box::new(move || {
                            picker.pick();
                            None
                        }));
                        let host = app_bridge.map(|bridge| {
                            bridge.listen(inbox.clone());
                            bridge.post_ready();
                            (bridge, crate::host::DirtyReport::default())
                        });
                        let reduced_motion = web_sys::window().and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten());

                        Ok(Box::new(Shell {
                            app,
                            inbox,
                            host,
                            reduced_motion,
                            doc_state: crate::doc_panel::DocState::default(),
                            show_doc_panel: false, // 初始折叠文档面板，只显示“文档”按钮 (Req 8)
                            doc_width: crate::buerligons::DOC_DEFAULT_WIDTH,
                            show_sidebar: true,    // 默认显示侧边栏 (Req 3)
                            sidebar_width: crate::buerligons::SIDEBAR_DEFAULT_WIDTH,
                            sidebar_state: crate::sidebar::SidebarState::default(),
                            font_data,
                            font_installed: false,
                        }))
                    }),
                )
                .await;

            if let (Err(e), Some(bridge)) = (&result, &bridge) {
                bridge.post_failed(&format!("{e:?}"));
            }
            if let Some(el) = document.get_element_by_id(LOADING_ID) {
                match result {
                    Ok(()) => el.remove(),
                    Err(e) => {
                        el.set_inner_html(&format!("<p>CADFlow failed to start: {e:?}</p><p>A browser with WebGPU or WebGL2 is required.</p>"))
                    }
                }
            }
        });
    }
}

/// True when the URL query (`?cpu&sample`) has a `cpu` flag.
#[cfg(any(target_arch = "wasm32", test))]
fn wants_cpu(query: &str) -> bool {
    query.trim_start_matches('?').split('&').any(|p| p == "cpu" || p.starts_with("cpu="))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("cadflow-web only runs in the browser: build it with `trunk build --release` in cadflow-web");
}
