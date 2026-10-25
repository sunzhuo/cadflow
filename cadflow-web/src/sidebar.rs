//! Left sidebar panel for CADFlow:
//! - Top tabs: Tool Sets (工具集), Layers (图层), Blocks (块), Groups (组)
//! - Default active tab: Tool Sets (工具集)
//! - Tab & subtab styling match original CADCraft Tool Sets style (t.tab_active & t.chrome, no blue lines)
//! - Complete Layers palette translated to Chinese
//! - Complete Properties inspector translated to Chinese with resizable height (Req 7)

use cadcraft_color::Color;
use cadcraft_doc::{EntityKind, Lineweight};
use cadcraft_ui_egui::CadApp;
use cadcraft_ui_egui::icons::{self, Icon};
use cadcraft_ui_egui::layers::swatch;
use cadcraft_ui_egui::palettes::{DRAFTING, MODELING};
use cadcraft_ui_egui::theme::Tokens;
use egui::{pos2, vec2, CursorIcon, Rect, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SidebarTopTab {
    #[default]
    ToolSets,
    Layers,
    Blocks,
    Groups,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarState {
    pub active_tab: SidebarTopTab,
    pub toolset_subtab: String,
    pub collapsed_categories: Vec<String>,
    pub properties_height: f32,
    pub show_layer_list: bool,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            active_tab: SidebarTopTab::ToolSets,
            toolset_subtab: "Drafting".to_string(),
            collapsed_categories: Vec::new(),
            properties_height: 280.0,
            show_layer_list: false,
        }
    }
}

impl SidebarState {
    pub fn ui(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let total_h = ui.available_height();

        // 限制特性面板高度并计算上下部分分配 (Req 7: 高度可调整)
        let prop_h = self.properties_height.clamp(100.0, (total_h - 160.0).max(100.0));
        self.properties_height = prop_h;
        let top_h = (total_h - prop_h - 6.0).max(120.0);

        // --------------------------------------------------------------------
        // 1. 上半部分：[工具集] [图层] [块] [组] 4个标签页
        // --------------------------------------------------------------------
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), top_h),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                // 顶部标签切换栏 (高度 28px，严格参考 CADCraft 工具集样式，无蓝色下划线)
                let tab_h = 28.0;
                let full_w = ui.available_width();
                let (tabs_rect, _) = ui.allocate_exact_size(vec2(full_w, tab_h), Sense::hover());
                let p = ui.painter();

                p.rect_filled(tabs_rect, 0.0, t.chrome);

                let tabs = [
                    (SidebarTopTab::ToolSets, "工具集"),
                    (SidebarTopTab::Layers, "图层"),
                    (SidebarTopTab::Blocks, "块"),
                    (SidebarTopTab::Groups, "组"),
                ];

                let tab_w = (full_w / tabs.len() as f32).floor();
                let mut clicked_tab = None;

                for (idx, (tab_variant, label)) in tabs.iter().enumerate() {
                    let tr = Rect::from_min_size(
                        pos2(tabs_rect.left() + idx as f32 * tab_w, tabs_rect.top()),
                        vec2(tab_w, tab_h),
                    );
                    let is_active = self.active_tab == *tab_variant;
                    let resp = ui.interact(tr, ui.id().with(("sb_tab", idx)), Sense::click());

                    // 使用原 toolset 样式：选中时为 tab_active，未选为 chrome；字体选中为 text，未选为 text_dim
                    p.rect_filled(tr, 0.0, if is_active { t.tab_active } else { t.chrome });
                    p.text(
                        tr.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        egui::FontId::proportional(13.0),
                        if is_active { t.text } else { t.text_dim },
                    );

                    if resp.clicked() {
                        clicked_tab = Some(*tab_variant);
                    }
                }

                if let Some(tab) = clicked_tab {
                    self.active_tab = tab;
                }

                // 分割线
                p.hline(tabs_rect.x_range(), tabs_rect.bottom(), Stroke::new(1.0, t.border));

                // 标签页内容
                match self.active_tab {
                    SidebarTopTab::ToolSets => self.render_toolsets(ui, app),
                    SidebarTopTab::Layers => self.render_layers(ui, app),
                    SidebarTopTab::Blocks => self.render_blocks(ui, app),
                    SidebarTopTab::Groups => self.render_groups(ui, app),
                }
            },
        );

        // --------------------------------------------------------------------
        // 2. 中间水平分割拖拽条 (Req 7: 调整特性面板高度)
        // --------------------------------------------------------------------
        let (sep_rect, sep_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::drag());
        let is_sep_hover = sep_resp.hovered() || sep_resp.dragged();

        if is_sep_hover {
            ui.ctx().set_cursor_icon(CursorIcon::ResizeVertical);
        }
        if sep_resp.dragged() {
            self.properties_height -= sep_resp.drag_delta().y;
        }

        ui.painter().rect_filled(
            sep_rect,
            0.0,
            if is_sep_hover { t.control_hover } else { t.chrome },
        );
        ui.painter().hline(
            sep_rect.x_range(),
            sep_rect.center().y,
            Stroke::new(1.0, if is_sep_hover { t.accent } else { t.border }),
        );

        // --------------------------------------------------------------------
        // 3. 下半部分：完整 CADCraft 特性面板 (Properties，翻译为中文)
        // --------------------------------------------------------------------
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), self.properties_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                self.render_properties(ui, app);
            },
        );
    }

    // ------------------------------------------------------------------------
    // 工具集内容渲染 (去除了折叠按钮，子标签样式恢复原 toolset 样式)
    // ------------------------------------------------------------------------
    fn render_toolsets(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();

        // 绘图 / 建模 子标签栏 (高度 26px，使用原 toolset 样式)
        let sub_h = 26.0;
        let full_w = ui.available_width();
        let (sub_rect, _) = ui.allocate_exact_size(vec2(full_w, sub_h), Sense::hover());
        let p = ui.painter();
        p.rect_filled(sub_rect, 0.0, t.chrome);

        let sub_w = 72.0;
        let mut sub_x = sub_rect.left() + 4.0;
        let subtabs = [("Drafting", "绘图"), ("Modeling", "建模")];
        let mut clicked_sub = None;

        for (id, label) in subtabs {
            let sub_tr = Rect::from_min_size(pos2(sub_x, sub_rect.top() + 1.0), vec2(sub_w, sub_h - 2.0));
            let is_active = self.toolset_subtab == id;
            let resp = ui.interact(sub_tr, ui.id().with(("ts_sub", id)), Sense::click());

            p.rect_filled(sub_tr, 0.0, if is_active { t.tab_active } else { t.chrome });
            p.text(
                sub_tr.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(12.5),
                if is_active { t.text } else { t.text_dim },
            );

            if resp.clicked() {
                clicked_sub = Some(id.to_string());
            }
            sub_x += sub_w + 4.0;
        }

        if let Some(sub) = clicked_sub {
            self.toolset_subtab = sub;
        }

        // 分割线
        p.hline(sub_rect.x_range(), sub_rect.bottom(), Stroke::new(1.0, t.border));

        // 工具列表滚动区域
        let groups = if self.toolset_subtab == "Drafting" {
            DRAFTING
        } else {
            MODELING
        };

        let mut start_cmd = None;
        egui::ScrollArea::vertical()
            .id_salt("sb_toolsets_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                for (name, large, small) in groups {
                    let collapsed = self.collapsed_categories.iter().any(|c| c == *name);

                    // 折叠分组标题行 (高度 22px)
                    let (hr, hresp) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::click());
                    let pp = ui.painter();
                    icons::paint(
                        pp,
                        Rect::from_center_size(pos2(hr.left() + 11.0, hr.center().y), vec2(11.0, 11.0)),
                        if collapsed { Icon::ChevronRight } else { Icon::ChevronDown },
                        false,
                    );
                    pp.text(
                        pos2(hr.left() + 22.0, hr.center().y),
                        egui::Align2::LEFT_CENTER,
                        cadcraft_ui_egui::i18n::t(name),
                        cadcraft_ui_egui::theme::body(),
                        t.text,
                    );
                    pp.hline(
                        (hr.left() + 18.0 + 80.0)..=hr.right() - 8.0,
                        hr.center().y,
                        Stroke::new(1.0, t.border.gamma_multiply(0.4)),
                    );

                    if hresp.clicked() {
                        if collapsed {
                            self.collapsed_categories.retain(|c| c != *name);
                        } else {
                            self.collapsed_categories.push(name.to_string());
                        }
                    }

                    if collapsed {
                        continue;
                    }

                    // 工具图标网格
                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), 0.0),
                        egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                        |ui| {
                            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);
                            ui.add_space(8.0);

                            for &(icon, id, label) in *large {
                                if icons::button(ui, icon, 28.0, cadcraft_ui_egui::i18n::t(label), false).clicked() {
                                    start_cmd = Some(id);
                                }
                            }
                            for &(icon, id, label) in *small {
                                if icons::button(ui, icon, 28.0, cadcraft_ui_egui::i18n::t(label), false).clicked() {
                                    start_cmd = Some(id);
                                }
                            }
                        },
                    );
                    ui.add_space(6.0);
                }
            });

        if let Some(cmd) = start_cmd {
            app.start(cmd);
        }
    }

    // ------------------------------------------------------------------------
    // 完整图层面板渲染 (Req 2: 沿用 CADCraft 原生 Layers 选项板与中文包)
    // ------------------------------------------------------------------------
    fn render_layers(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let mut cmd_to_start = None;
        let mut cmd_to_run = None;

        egui::ScrollArea::vertical()
            .id_salt("sb_layers_full_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);

                // 1. 图层操作图标行 (8个工具按钮 + 解锁)
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
                    let tools = [
                        (Icon::LayerProps, "ui.dialog.layers", "Layer Properties"),
                        (Icon::MakeCurrent, "laymcur", "Make Object's Layer Current"),
                        (Icon::LayerMatch, "laymch", "Match Layer"),
                        (Icon::LayerPrev, "layerp", "Previous Layer"),
                        (Icon::LayerIso, "layiso", "Isolate"),
                        (Icon::LayerOff, "layoff", "Turn Off Layer"),
                        (Icon::LayerFreeze, "layfrz", "Freeze Layer"),
                        (Icon::LayerLock, "laylck", "Lock Layer"),
                        (Icon::Unlock, "layulk", "Unlock Layer"),
                    ];
                    for (icon, c, tip) in tools {
                        if icons::button(ui, icon, 24.0, cadcraft_ui_egui::i18n::t(tip), false).clicked() {
                            if c.starts_with("ui.") {
                                cmd_to_start = Some(c);
                            } else {
                                cmd_to_run = Some((c, json!({})));
                            }
                        }
                    }
                });

                ui.add_space(6.0);

                // 2. 当前活动图层下拉行 (Current layer combo)
                let Ok(d) = app.session.doc() else { return };
                let cur = d.header.str("CLAYER", "0");
                let layers: Vec<(String, bool, bool, bool, Color)> = d
                    .layers
                    .iter()
                    .map(|l| (l.name.clone(), l.on, l.frozen, l.locked, l.color))
                    .collect();
                let cur_info = layers.iter().find(|l| l.0.eq_ignore_ascii_case(&cur)).cloned();

                let mut set_current = None;
                let mut toggle: Option<(String, &'static str, bool)> = None;

                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let w = ui.available_width() - 10.0;
                    let (rect, resp) = ui.allocate_exact_size(vec2(w, 24.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(rect, 3.0, t.control);

                    if let Some((name, on, frozen, locked, color)) = &cur_info {
                        let mut x = rect.left() + 6.0;
                        for icon in [
                            if *on { Icon::Bulb } else { Icon::BulbOff },
                            if *locked { Icon::Lock } else { Icon::Unlock },
                            if *frozen { Icon::Snowflake } else { Icon::Sun },
                        ] {
                            icons::paint(p, Rect::from_min_size(pos2(x, rect.top() + 4.0), vec2(16.0, 16.0)), icon, false);
                            x += 18.0;
                        }
                        swatch(p, Rect::from_min_size(pos2(x + 2.0, rect.top() + 7.0), vec2(10.0, 10.0)), *color);
                        p.text(pos2(x + 18.0, rect.center().y), egui::Align2::LEFT_CENTER, name, cadcraft_ui_egui::theme::body(), t.text);
                    }
                    icons::paint(p, Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);

                    egui::Popup::from_toggle_button_response(&resp)
                        .width(w)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            for (name, on, frozen, locked, color) in &layers {
                                ui.horizontal(|ui| {
                                    if icons::button(ui, if *on { Icon::Bulb } else { Icon::BulbOff }, 18.0, "开/关", false).clicked() {
                                        toggle = Some((name.clone(), "on", !*on));
                                    }
                                    if icons::button(ui, if *frozen { Icon::Snowflake } else { Icon::Sun }, 18.0, "冻结/解冻", false).clicked() {
                                        toggle = Some((name.clone(), "frozen", !*frozen));
                                    }
                                    if icons::button(ui, if *locked { Icon::Lock } else { Icon::Unlock }, 18.0, "锁定/解锁", false).clicked() {
                                        toggle = Some((name.clone(), "locked", !*locked));
                                    }
                                    let (sr, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                                    swatch(ui.painter(), sr, *color);
                                    if ui.selectable_label(name.eq_ignore_ascii_case(&cur), name).clicked() {
                                        set_current = Some(name.clone());
                                    }
                                });
                            }
                        });
                });

                ui.add_space(4.0);

                // 3. 图层状态框
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let w = ui.available_width() - 10.0;
                    let (rect, _) = ui.allocate_exact_size(vec2(w, 24.0), Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, t.control);
                    ui.painter().text(
                        pos2(rect.left() + 8.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        cadcraft_ui_egui::i18n::t("Unsaved Layer State"),
                        cadcraft_ui_egui::theme::body(),
                        t.text,
                    );
                    icons::paint(ui.painter(), Rect::from_center_size(pos2(rect.right() - 12.0, rect.center().y), vec2(12.0, 12.0)), Icon::ChevronDown, false);
                });

                ui.add_space(4.0);

                // 4. 显示/隐藏图层列表折叠项
                let label = if self.show_layer_list { "Hide Layer List" } else { "Show Layer List" };
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let (rect, resp) = ui.allocate_exact_size(vec2(180.0, 22.0), Sense::click());
                    icons::paint(
                        ui.painter(),
                        Rect::from_center_size(pos2(rect.left() + 7.0, rect.center().y), vec2(11.0, 11.0)),
                        if self.show_layer_list { Icon::ChevronDown } else { Icon::ChevronRight },
                        false,
                    );
                    ui.painter().text(
                        pos2(rect.left() + 20.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        cadcraft_ui_egui::i18n::t(label),
                        cadcraft_ui_egui::theme::body(),
                        t.text,
                    );
                    if resp.clicked() {
                        self.show_layer_list = !self.show_layer_list;
                    }
                });

                // 展开的图层列表
                if self.show_layer_list {
                    ui.add_space(2.0);
                    for (name, on, frozen, locked, color) in &layers {
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            if icons::button(ui, if *on { Icon::Bulb } else { Icon::BulbOff }, 18.0, "开/关", false).clicked() {
                                toggle = Some((name.clone(), "on", !*on));
                            }
                            if icons::button(ui, if *frozen { Icon::Snowflake } else { Icon::Sun }, 18.0, "冻结/解冻", false).clicked() {
                                toggle = Some((name.clone(), "frozen", !*frozen));
                            }
                            if icons::button(ui, if *locked { Icon::Lock } else { Icon::Unlock }, 18.0, "锁定/解锁", false).clicked() {
                                toggle = Some((name.clone(), "locked", !*locked));
                            }
                            let (sr, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                            swatch(ui.painter(), sr, *color);
                            let r = ui.selectable_label(name.eq_ignore_ascii_case(&cur), name);
                            if r.clicked() {
                                set_current = Some(name.clone());
                            }
                        });
                    }
                }

                if let Some(n) = set_current {
                    let _ = app.run("layer.current", json!({ "name": n }));
                }
                if let Some((n, k, v)) = toggle {
                    let _ = app.run("layer.set", json!({ "name": n, k: v }));
                }
            });

        if let Some(cmd) = cmd_to_start {
            app.start(cmd);
        }
        if let Some((c, p)) = cmd_to_run {
            let _ = app.run(c, p);
        }
    }

    // ------------------------------------------------------------------------
    // 块面板渲染
    // ------------------------------------------------------------------------
    fn render_blocks(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let _t = Tokens::get();
        let mut to_insert = None;

        egui::ScrollArea::vertical()
            .id_salt("sb_blocks_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);
                let Ok(d) = app.session.doc() else { return };
                let names: Vec<String> = d.blocks.keys().filter(|k| !k.starts_with('*')).cloned().collect();

                if names.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(cadcraft_ui_egui::i18n::t("No blocks defined in this drawing."));
                    });
                } else {
                    for n in names {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                            icons::paint(ui.painter(), r, Icon::Blocks, false);
                            if ui.button(&n).on_hover_text(cadcraft_ui_egui::i18n::t("Insert this block")).clicked() {
                                to_insert = Some(n);
                            }
                        });
                    }
                }
            });

        if let Some(n) = to_insert {
            let _ = app.run("insert", json!({ "name": n }));
        }
    }

    // ------------------------------------------------------------------------
    // 组面板渲染
    // ------------------------------------------------------------------------
    fn render_groups(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let mut cmd = None;
        egui::ScrollArea::vertical()
            .id_salt("sb_groups_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    if ui.button(cadcraft_ui_egui::i18n::t("Group")).clicked() {
                        cmd = Some("group");
                    }
                    if ui.button(cadcraft_ui_egui::i18n::t("Ungroup")).clicked() {
                        cmd = Some("ungroup");
                    }
                    if ui.button(cadcraft_ui_egui::i18n::t("Group Edit")).clicked() {
                        cmd = Some("groupedit");
                    }
                });
            });

        if let Some(c) = cmd {
            app.start(c);
        }
    }

    // ------------------------------------------------------------------------
    // 完整特性面板渲染 (Req 2: 沿用 CADCraft 原生 Properties 选项板全部控件并翻译为中文)
    // ------------------------------------------------------------------------
    fn render_properties(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let sel = app.session.selection();
        let Ok(d) = app.session.doc() else { return };
        let h = &d.header;

        let lu = h.i64("LUNITS", 2);
        let lp = h.i64("LUPREC", 4);
        let len = |v: f64| cadcraft_engine::units::format_distance(v, lu, lp);

        let mut run_cmd: Option<(&'static str, Value)> = None;
        let mut start_cmd: Option<&'static str> = None;

        // 特性头部行 (标题 + 全选/我的 + 选择集类型下拉 + PickAdd + Select + QuickSelect)
        let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
        ui.painter().rect_filled(hr, 0.0, t.chrome);
        ui.painter().text(
            pos2(hr.left() + 8.0, hr.center().y),
            egui::Align2::LEFT_CENTER,
            cadcraft_ui_egui::i18n::t("Properties"),
            cadcraft_ui_egui::theme::body(),
            t.text,
        );

        // Segmented control: 全部 | 我的
        let seg = Rect::from_min_size(pos2(hr.right() - 110.0, hr.top() + 4.0), vec2(60.0, 18.0));
        ui.painter().rect_filled(seg, 3.0, t.chrome_dark);
        let half = Rect::from_min_size(seg.min, vec2(30.0, 18.0));
        let my = half.translate(vec2(30.0, 0.0));
        ui.painter().rect_filled(if app.ui.properties_all { half } else { my }, 3.0, t.control);
        ui.painter().text(half.center(), egui::Align2::CENTER_CENTER, cadcraft_ui_egui::i18n::t("All"), cadcraft_ui_egui::theme::small(), t.text);
        ui.painter().text(my.center(), egui::Align2::CENTER_CENTER, cadcraft_ui_egui::i18n::t("My"), cadcraft_ui_egui::theme::small(), t.text_faint);

        // 选择集摘要与工具栏按钮
        let mut types: Vec<(&'static str, Vec<String>)> = Vec::new();
        for s in &sel {
            if let Some(e) = d.entity(*s) {
                let n = e.kind.type_name();
                match types.iter_mut().find(|(ty, _)| *ty == n) {
                    Some((_, hs)) => hs.push(s.hex()),
                    None => types.push((n, vec![s.hex()])),
                }
            }
        }
        let count: usize = types.iter().map(|(_, hs)| hs.len()).sum();
        let header_label = match (count, types.as_slice()) {
            (0, _) => cadcraft_ui_egui::i18n::t("No selection").to_string(),
            (1, [(n, _)]) => cadcraft_ui_egui::i18n::t(n).to_string(),
            (n, [(ty, _)]) => format!("{} ({n})", cadcraft_ui_egui::i18n::t(ty)),
            (n, _) => format!("{} ({n})", cadcraft_ui_egui::i18n::t("All")),
        };

        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.spacing_mut().item_spacing = vec2(3.0, 0.0);
            let w = (ui.available_width() - 84.0).max(60.0);
            let (rect, resp) = ui.allocate_exact_size(vec2(w, 20.0), Sense::click());
            ui.painter().rect_filled(rect, 3.0, t.control);
            ui.painter().text(pos2(rect.left() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, &header_label, cadcraft_ui_egui::theme::small(), t.text);

            if count > 0 {
                egui::Popup::from_toggle_button_response(&resp)
                    .width(w - 10.0)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                    .show(|ui| {
                        for (ty, hs) in &types {
                            if ui.button(format!("{} ({})", cadcraft_ui_egui::i18n::t(ty), hs.len())).clicked() {
                                run_cmd = Some(("select", json!({ "handles": hs })));
                            }
                        }
                    });
            }

            let pickadd = app.session.settings.pickadd;
            let tip = if pickadd {
                cadcraft_ui_egui::i18n::t("Toggle PICKADD (on: picks add to the selection)")
            } else {
                cadcraft_ui_egui::i18n::t("Toggle PICKADD (off: each pick replaces the selection)")
            };
            if icons::button(ui, Icon::PickAdd, 22.0, tip, pickadd).clicked() {
                run_cmd = Some(("setvar", json!({ "name": "PICKADD", "value": i32::from(!pickadd) })));
            }
            if icons::button(ui, Icon::SelectObjects, 22.0, cadcraft_ui_egui::i18n::t("Select Objects"), false).clicked() {
                start_cmd = Some("select");
            }
            if icons::button(ui, Icon::QuickSelect, 22.0, cadcraft_ui_egui::i18n::t("Quick Select"), false).clicked() {
                start_cmd = Some("ui.dialog.qselect");
            }
        });

        ui.add_space(4.0);

        // 滚动属性行
        egui::ScrollArea::vertical()
            .id_salt("sb_properties_rows_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if sel.is_empty() {
                    // 无选择对象时展示当前图层与默认属性
                    let layers: Vec<String> = d.layers.iter().map(|l| l.name.clone()).collect();
                    let linetypes: Vec<String> = d.linetypes.iter().map(|l| l.name.clone()).collect();
                    let text_styles: Vec<String> = d.text_styles.iter().map(|l| l.name.clone()).collect();
                    let dim_styles: Vec<String> = d.dim_styles.iter().map(|l| l.name.clone()).collect();

                    self.prop_row_custom(ui, "Color", |ui| {
                        ui.menu_button(format!("■ {}", cadcraft_ui_egui::i18n::t(&Color::from_aci(h.i64("CECOLOR", 256) as i16).name())), |ui| {
                            for (n, c) in [
                                ("ByLayer", 256),
                                ("ByBlock", 0),
                                ("Red", 1),
                                ("Yellow", 2),
                                ("Green", 3),
                                ("Cyan", 4),
                                ("Blue", 5),
                                ("Magenta", 6),
                                ("White", 7),
                            ] {
                                if ui.button(cadcraft_ui_egui::i18n::t(n)).clicked() {
                                    run_cmd = Some(("color", json!({ "color": c })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Layer", |ui| {
                        let cur_l = h.str("CLAYER", "0");
                        ui.menu_button(&cur_l, |ui| {
                            for l in &layers {
                                if ui.button(l).clicked() {
                                    run_cmd = Some(("layer.current", json!({ "name": l })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Linetype", |ui| {
                        let cur_lt = h.str("CELTYPE", "ByLayer");
                        ui.menu_button(cadcraft_ui_egui::i18n::t(&cur_lt), |ui| {
                            for l in ["ByLayer", "ByBlock"].iter().copied().chain(linetypes.iter().map(|s| s.as_str())) {
                                if ui.button(cadcraft_ui_egui::i18n::t(l)).clicked() {
                                    run_cmd = Some(("linetype", json!({ "current": l })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Linetype scale", |ui| {
                        let mut val = format!("{:.4}", h.f64("CELTSCALE", 1.0));
                        if ui.text_edit_singleline(&mut val).lost_focus()
                            && let Ok(f) = val.parse::<f64>()
                        {
                            run_cmd = Some(("setvar", json!({ "name": "CELTSCALE", "value": f })));
                        }
                    });

                    self.prop_row_custom(ui, "Lineweight", |ui| {
                        let lw_name = Lineweight::from_dxf(h.i64("CELWEIGHT", -1) as i16).name();
                        ui.menu_button(cadcraft_ui_egui::i18n::t(&lw_name), |ui| {
                            for n in ["ByLayer", "ByBlock", "Default"] {
                                if ui.button(cadcraft_ui_egui::i18n::t(n)).clicked() {
                                    run_cmd = Some(("lweight", json!({ "lineweight": n })));
                                    ui.close();
                                }
                            }
                            for v in Lineweight::STANDARD {
                                if ui.button(format!("{:.2} mm", f64::from(v) / 100.0)).clicked() {
                                    run_cmd = Some(("lweight", json!({ "lineweight": f64::from(v) / 100.0 })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Transparency", |ui| {
                        ui.label(cadcraft_ui_egui::i18n::t("ByLayer"));
                    });

                    self.prop_row_custom(ui, "Thickness", |ui| {
                        let mut val = len(h.f64("THICKNESS", 0.0));
                        if ui.text_edit_singleline(&mut val).lost_focus()
                            && let Ok(f) = val.parse::<f64>()
                        {
                            run_cmd = Some(("setvar", json!({ "name": "THICKNESS", "value": f })));
                        }
                    });

                    self.prop_row_custom(ui, "Text style", |ui| {
                        let cur_ts = h.str("TEXTSTYLE", "Standard");
                        ui.menu_button(&cur_ts, |ui| {
                            for s in &text_styles {
                                if ui.button(s).clicked() {
                                    run_cmd = Some(("style.current", json!({ "name": s })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Dimension style", |ui| {
                        let cur_ds = h.str("DIMSTYLE", "Standard");
                        ui.menu_button(&cur_ds, |ui| {
                            for s in &dim_styles {
                                if ui.button(s).clicked() {
                                    run_cmd = Some(("dimstyle.current", json!({ "name": s })));
                                    ui.close();
                                }
                            }
                        });
                    });

                    self.prop_row_custom(ui, "Annotation scale", |ui| {
                        ui.label("1:1");
                    });

                    self.prop_row_custom(ui, "Text height", |ui| {
                        let mut val = len(h.f64("TEXTSIZE", 0.2));
                        if ui.text_edit_singleline(&mut val).lost_focus()
                            && let Ok(f) = val.parse::<f64>()
                        {
                            run_cmd = Some(("setvar", json!({ "name": "TEXTSIZE", "value": f })));
                        }
                    });

                    self.prop_row_custom(ui, "Plot style", |ui| {
                        ui.label(cadcraft_ui_egui::i18n::t("ByColor"));
                    });
                    self.prop_row_custom(ui, "Plot style table", |ui| {
                        ui.label(cadcraft_ui_egui::i18n::t("None"));
                    });
                } else {
                    // 选中图元时的常规属性与几何属性
                    let _ids: Vec<String> = sel.iter().map(|s| s.hex()).collect();
                    let first = sel.first().and_then(|s| d.entity(*s)).map(|e| (**e).clone());

                    if let Some(e) = first {
                        // 常规分组
                        self.section_divider(ui, "General");

                        self.prop_row_custom(ui, "Color", |ui| {
                            ui.label(cadcraft_ui_egui::i18n::t(&e.common.color.name()));
                        });
                        self.prop_row_custom(ui, "Layer", |ui| {
                            ui.label(&e.common.layer);
                        });
                        self.prop_row_custom(ui, "Linetype", |ui| {
                            ui.label(cadcraft_ui_egui::i18n::t(&e.common.linetype));
                        });
                        self.prop_row_custom(ui, "Linetype scale", |ui| {
                            ui.label(format!("{:.4}", e.common.ltscale));
                        });
                        self.prop_row_custom(ui, "Lineweight", |ui| {
                            ui.label(cadcraft_ui_egui::i18n::t(&e.common.lineweight.name()));
                        });

                        // 几何分组 (单选图元时展示具体几何参数)
                        if sel.len() == 1 {
                            self.section_divider(ui, "Geometry");
                            match &e.kind {
                                EntityKind::Line(l) => {
                                    self.prop_row_custom(ui, "Start X", |ui| { ui.label(len(l.a.x)); });
                                    self.prop_row_custom(ui, "Start Y", |ui| { ui.label(len(l.a.y)); });
                                    self.prop_row_custom(ui, "End X", |ui| { ui.label(len(l.b.x)); });
                                    self.prop_row_custom(ui, "End Y", |ui| { ui.label(len(l.b.y)); });
                                    let dlt = l.b.xy() - l.a.xy();
                                    self.prop_row_custom(ui, "Delta X", |ui| { ui.label(len(dlt.x)); });
                                    self.prop_row_custom(ui, "Delta Y", |ui| { ui.label(len(dlt.y)); });
                                    self.prop_row_custom(ui, "Length", |ui| { ui.label(len(dlt.len())); });
                                }
                                EntityKind::Circle(c) => {
                                    self.prop_row_custom(ui, "Center X", |ui| { ui.label(len(c.center.x)); });
                                    self.prop_row_custom(ui, "Center Y", |ui| { ui.label(len(c.center.y)); });
                                    self.prop_row_custom(ui, "Radius", |ui| { ui.label(len(c.radius)); });
                                    self.prop_row_custom(ui, "Diameter", |ui| { ui.label(len(c.radius * 2.0)); });
                                    self.prop_row_custom(ui, "Circumference", |ui| { ui.label(len(c.radius * std::f64::consts::TAU)); });
                                    self.prop_row_custom(ui, "Area", |ui| { ui.label(format!("{:.4}", c.radius * c.radius * std::f64::consts::PI)); });
                                }
                                EntityKind::Arc(a) => {
                                    self.prop_row_custom(ui, "Radius", |ui| { ui.label(len(a.radius)); });
                                    self.prop_row_custom(ui, "Start angle", |ui| { ui.label(format!("{:.2}°", a.start.to_degrees())); });
                                    self.prop_row_custom(ui, "End angle", |ui| { ui.label(format!("{:.2}°", a.end.to_degrees())); });
                                }
                                EntityKind::Text(tx) => {
                                    self.prop_row_custom(ui, "Contents", |ui| { ui.label(&tx.value); });
                                    self.prop_row_custom(ui, "Height", |ui| { ui.label(len(tx.height)); });
                                }
                                EntityKind::MText(tx) => {
                                    self.prop_row_custom(ui, "Contents", |ui| { ui.label(&tx.contents); });
                                }
                                _ => {
                                    self.prop_row_custom(ui, "Type", |ui| { ui.label(cadcraft_ui_egui::i18n::t(e.kind.type_name())); });
                                }
                            }
                        }
                    }
                }
            });

        if let Some((c, p)) = run_cmd {
            let _ = app.run(c, p);
        }
        if let Some(c) = start_cmd {
            app.start(c);
        }
    }

    fn section_divider(&self, ui: &mut Ui, title: &str) {
        let t = Tokens::get();
        let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
        ui.painter().rect_filled(r.shrink2(vec2(4.0, 1.0)), 2.0, t.chrome_dark);
        ui.painter().text(
            pos2(r.left() + 10.0, r.center().y),
            egui::Align2::LEFT_CENTER,
            cadcraft_ui_egui::i18n::t(title),
            cadcraft_ui_egui::theme::small(),
            t.text_dim,
        );
    }

    fn prop_row_custom(&self, ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
        let t = Tokens::get();
        ui.horizontal(|ui| {
            let mut job = egui::text::LayoutJob::simple(
                cadcraft_ui_egui::i18n::t(label).to_string(),
                cadcraft_ui_egui::theme::body(),
                t.text_dim,
                96.0,
            );
            job.halign = egui::Align::RIGHT;
            let galley = ui.painter().layout_job(job);
            let (lr, _) = ui.allocate_exact_size(vec2(104.0, galley.size().y.max(22.0)), Sense::hover());
            ui.painter().galley(pos2(lr.right() - 6.0, lr.center().y - galley.size().y / 2.0), galley, t.text_dim);
            add(ui);
        });
    }
}
