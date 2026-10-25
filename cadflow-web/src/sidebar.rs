//! Left sidebar panel for CADFlow:
//! - Top tabs: Tool Sets (工具集), Layers (图层), Blocks (块), Groups (组)
//! - Default active tab: Tool Sets (工具集)
//! - Bottom section: Properties (特性) inspector
//! - No "Collapse Tool Sets" button in Tool Sets

use cadcraft_engine::doc::{EntityKind, Handle};
use cadcraft_ui_egui::CadApp;
use cadcraft_ui_egui::icons::{self, Icon};
use cadcraft_ui_egui::palettes::{DRAFTING, MODELING};
use cadcraft_ui_egui::theme::Tokens;
use egui::{pos2, vec2, CornerRadius, Rect, Sense, Stroke, Ui};
use serde::{Deserialize, Serialize};
use serde_json::json;

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
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            active_tab: SidebarTopTab::ToolSets,
            toolset_subtab: "Drafting".to_string(),
            collapsed_categories: Vec::new(),
        }
    }
}

impl SidebarState {
    pub fn ui(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let total_height = ui.available_height();
        let top_height = (total_height * 0.58).max(180.0);
        let bottom_height = (total_height - top_height - 6.0).max(120.0);

        // --------------------------------------------------------------------
        // 1. 上半部分：[工具集] [图层] [块] [组] 4个标签页
        // --------------------------------------------------------------------
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), top_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                // 顶部标签切换栏 (高度 30px)
                let tab_h = 30.0;
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

                    if is_active {
                        p.rect_filled(tr, 0.0, t.tab_active);
                        let ind = Rect::from_min_max(
                            pos2(tr.left() + 4.0, tr.bottom() - 2.5),
                            pos2(tr.right() - 4.0, tr.bottom()),
                        );
                        p.rect_filled(ind, CornerRadius::same(1), t.accent);
                    }

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

                ui.add_space(2.0);
                match self.active_tab {
                    SidebarTopTab::ToolSets => self.render_toolsets(ui, app),
                    SidebarTopTab::Layers => self.render_layers(ui, app),
                    SidebarTopTab::Blocks => self.render_blocks(ui, app),
                    SidebarTopTab::Groups => self.render_groups(ui, app),
                }
            },
        );

        // 分割线
        ui.add_space(2.0);
        let (sep_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 2.0), Sense::hover());
        ui.painter().hline(sep_rect.x_range(), sep_rect.center().y, Stroke::new(1.0, t.border));
        ui.add_space(2.0);

        // --------------------------------------------------------------------
        // 2. 下半部分：[特性] 面板
        // --------------------------------------------------------------------
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), bottom_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                self.render_properties(ui, app);
            },
        );
    }

    // ------------------------------------------------------------------------
    // 工具集内容：无“折叠工具集”按钮
    // ------------------------------------------------------------------------
    fn render_toolsets(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();

        // 绘图 / 建模 子切换按钮
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            let is_drafting = self.toolset_subtab == "Drafting";
            if ui.selectable_label(is_drafting, "绘图").clicked() {
                self.toolset_subtab = "Drafting".to_string();
                app.ui.toolset_tab = "Drafting".to_string();
            }
            let is_modeling = self.toolset_subtab == "Modeling";
            if ui.selectable_label(is_modeling, "建模").clicked() {
                self.toolset_subtab = "Modeling".to_string();
                app.ui.toolset_tab = "Modeling".to_string();
            }
        });
        ui.add_space(2.0);

        let groups = if self.toolset_subtab == "Drafting" { DRAFTING } else { MODELING };
        let mut clicked_cmd = None;
        let mut toggle_cat = None;

        egui::ScrollArea::vertical()
            .id_salt("sidebar_toolsets_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                for (name, large, small) in groups {
                    let collapsed = self.collapsed_categories.iter().any(|c| c == name);

                    let (hr, hresp) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
                    let pp = ui.painter();
                    if hresp.hovered() {
                        pp.rect_filled(hr, CornerRadius::same(3), t.control_hover.gamma_multiply(0.4));
                    }

                    icons::paint(
                        pp,
                        Rect::from_center_size(pos2(hr.left() + 11.0, hr.center().y), vec2(11.0, 11.0)),
                        if collapsed { Icon::ChevronRight } else { Icon::ChevronDown },
                        false,
                    );

                    let translated_title = cadcraft_ui_egui::i18n::t(name);
                    pp.text(
                        pos2(hr.left() + 22.0, hr.center().y),
                        egui::Align2::LEFT_CENTER,
                        translated_title,
                        egui::FontId::proportional(12.5),
                        t.text,
                    );

                    if hresp.clicked() {
                        toggle_cat = Some(name.to_string());
                    }

                    if !collapsed {
                        ui.add_space(4.0);

                        if !large.is_empty() {
                            ui.horizontal_wrapped(|ui| {
                                ui.add_space(6.0);
                                ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
                                for (icon, cmd, tip) in large.iter() {
                                    let translated_tip = cadcraft_ui_egui::i18n::t(tip);
                                    let resp = icons::button(ui, *icon, 38.0, translated_tip, false);
                                    if resp.clicked() {
                                        clicked_cmd = Some(*cmd);
                                    }
                                }
                            });
                        }

                        if !small.is_empty() {
                            ui.add_space(2.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.add_space(6.0);
                                ui.spacing_mut().item_spacing = vec2(3.0, 3.0);
                                for (icon, cmd, tip) in small.iter() {
                                    let translated_tip = cadcraft_ui_egui::i18n::t(tip);
                                    let resp = icons::button(ui, *icon, 23.0, translated_tip, false);
                                    if resp.clicked() {
                                        clicked_cmd = Some(*cmd);
                                    }
                                }
                            });
                        }

                        ui.add_space(6.0);
                    }

                    let (sr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                    ui.painter().hline(sr.x_range().shrink(6.0), sr.center().y, Stroke::new(1.0, t.border));
                }
            });

        if let Some(cat) = toggle_cat {
            if let Some(pos) = self.collapsed_categories.iter().position(|c| c == &cat) {
                self.collapsed_categories.remove(pos);
            } else {
                self.collapsed_categories.push(cat);
            }
        }

        if let Some(cmd) = clicked_cmd {
            app.start(cmd);
        }
    }

    // ------------------------------------------------------------------------
    // 图层内容
    // ------------------------------------------------------------------------
    fn render_layers(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let mut start_cmd = None;

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            if ui.small_button("＋ 新建图层").on_hover_text("创建新图层").clicked() {
                start_cmd = Some("layer");
            }
            if ui.small_button("⚙ 图层管理器").on_hover_text("打开图层特性管理器").clicked() {
                start_cmd = Some("ui.dialog.layers");
            }
        });
        ui.add_space(4.0);

        if let Some(cmd) = start_cmd {
            app.start(cmd);
        }

        let Ok(d) = app.session.doc() else {
            ui.label(egui::RichText::new("无打开的图形").color(t.text_dim));
            return;
        };

        let current_layer = d.header.str("CLAYER", "0");
        let layers: Vec<_> = d.layers.iter().map(|l| (l.name.clone(), l.on, l.frozen, l.locked, l.color)).collect();

        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("当前图层: {current_layer}")).strong().color(t.text));
        });
        ui.add_space(2.0);

        let mut toggle = None;
        let mut set_current = None;

        egui::ScrollArea::vertical()
            .id_salt("sidebar_layers_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);

                for (name, on, frozen, locked, color) in layers {
                    let is_cur = name.eq_ignore_ascii_case(&current_layer);
                    let row_w = ui.available_width();
                    let (row_rect, row_resp) = ui.allocate_exact_size(vec2(row_w, 24.0), Sense::click());

                    if is_cur {
                        ui.painter().rect_filled(row_rect, CornerRadius::same(3), t.control);
                    } else if row_resp.hovered() {
                        ui.painter().rect_filled(row_rect, CornerRadius::same(3), t.control_hover.gamma_multiply(0.3));
                    }

                    // 开关 (Bulb)
                    let bulb_rect = Rect::from_center_size(pos2(row_rect.left() + 12.0, row_rect.center().y), vec2(16.0, 16.0));
                    let bulb_resp = ui.interact(bulb_rect, ui.id().with(("layer_bulb", &name)), Sense::click());
                    icons::paint(ui.painter(), bulb_rect, if on { Icon::Bulb } else { Icon::BulbOff }, false);
                    if bulb_resp.clicked() {
                        toggle = Some((name.clone(), "on", !on));
                    }

                    // 冻结 (Snowflake / Sun)
                    let freeze_rect = Rect::from_center_size(pos2(row_rect.left() + 30.0, row_rect.center().y), vec2(16.0, 16.0));
                    let freeze_resp = ui.interact(freeze_rect, ui.id().with(("layer_freeze", &name)), Sense::click());
                    icons::paint(ui.painter(), freeze_rect, if frozen { Icon::Snowflake } else { Icon::Sun }, false);
                    if freeze_resp.clicked() {
                        toggle = Some((name.clone(), "frozen", !frozen));
                    }

                    // 锁定 (Lock / Unlock)
                    let lock_rect = Rect::from_center_size(pos2(row_rect.left() + 48.0, row_rect.center().y), vec2(16.0, 16.0));
                    let lock_resp = ui.interact(lock_rect, ui.id().with(("layer_lock", &name)), Sense::click());
                    icons::paint(ui.painter(), lock_rect, if locked { Icon::Lock } else { Icon::Unlock }, false);
                    if lock_resp.clicked() {
                        toggle = Some((name.clone(), "locked", !locked));
                    }

                    // 颜色色块
                    let swatch_rect = Rect::from_center_size(pos2(row_rect.left() + 66.0, row_rect.center().y), vec2(12.0, 12.0));
                    cadcraft_ui_egui::layers::swatch(ui.painter(), swatch_rect, color);

                    // 图层名称
                    let text_pos = pos2(row_rect.left() + 78.0, row_rect.center().y);
                    ui.painter().text(
                        text_pos,
                        egui::Align2::LEFT_CENTER,
                        &name,
                        egui::FontId::proportional(12.5),
                        if is_cur { t.text } else { t.text_dim },
                    );

                    if row_resp.clicked() {
                        set_current = Some(name.clone());
                    }
                }
            });

        if let Some((n, k, v)) = toggle {
            let _ = app.run("layer.set", json!({ "name": n, k: v }));
        }
        if let Some(n) = set_current {
            let _ = app.run("layer.current", json!({ "name": n }));
        }
    }

    // ------------------------------------------------------------------------
    // 块面板内容
    // ------------------------------------------------------------------------
    fn render_blocks(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let mut start_cmd = None;

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            if ui.small_button("＋ 创建块").on_hover_text("从选择的图元创建块定义").clicked() {
                start_cmd = Some("block");
            }
            if ui.small_button("📥 插入块").on_hover_text("插入块引用").clicked() {
                start_cmd = Some("insert");
            }
        });
        ui.add_space(4.0);

        if let Some(cmd) = start_cmd {
            app.start(cmd);
        }

        let Ok(d) = app.session.doc() else {
            ui.label(egui::RichText::new("无打开的图形").color(t.text_dim));
            return;
        };

        let block_names: Vec<(String, usize)> = d
            .blocks
            .iter()
            .filter(|(k, _)| !k.starts_with('*'))
            .map(|(k, b)| (k.clone(), b.entities.len()))
            .collect();

        if block_names.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("当前图形中无块定义").size(12.0).color(t.text_dim));
            });
            return;
        }

        let mut insert_target = None;
        let mut edit_target = None;

        egui::ScrollArea::vertical()
            .id_salt("sidebar_blocks_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (name, count) in block_names {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        let (icon_rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                        icons::paint(ui.painter(), icon_rect, Icon::Blocks, false);

                        ui.label(egui::RichText::new(&name).strong().color(t.text));
                        ui.label(egui::RichText::new(format!("({count} 图元)")).size(11.0).color(t.text_dim));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("编辑").on_hover_text("编辑此块定义").clicked() {
                                edit_target = Some(name.clone());
                            }
                            if ui.small_button("插入").on_hover_text("插入此块引用").clicked() {
                                insert_target = Some(name.clone());
                            }
                        });
                    });
                    ui.separator();
                }
            });

        if let Some(name) = insert_target {
            let _ = app.run("insert", json!({ "name": name }));
        }
        if let Some(name) = edit_target {
            let _ = app.run("bedit", json!({ "name": name }));
        }
    }

    // ------------------------------------------------------------------------
    // 组面板内容
    // ------------------------------------------------------------------------
    fn render_groups(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let mut start_cmd = None;

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            if ui.small_button("＋ 编组").on_hover_text("将所选图元编成一个组 (GROUP)").clicked() {
                start_cmd = Some("group");
            }
            if ui.small_button("✕ 解除编组").on_hover_text("解除当前编组 (UNGROUP)").clicked() {
                start_cmd = Some("ungroup");
            }
        });
        ui.add_space(4.0);

        if let Some(cmd) = start_cmd {
            app.start(cmd);
        }

        let Ok(d) = app.session.doc() else {
            ui.label(egui::RichText::new("无打开的图形").color(t.text_dim));
            return;
        };

        let groups: Vec<(String, String, usize, Vec<Handle>)> = d
            .groups
            .iter()
            .map(|g| (g.name.clone(), g.description.clone(), g.members.len(), g.members.clone()))
            .collect();

        if groups.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("当前图形中无编组").size(12.0).color(t.text_dim));
            });
            return;
        }

        let mut ungroup_target = None;
        let mut select_target = None;

        egui::ScrollArea::vertical()
            .id_salt("sidebar_groups_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (name, desc, count, members) in groups {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&name).strong().color(t.text));
                            if !desc.is_empty() {
                                ui.label(egui::RichText::new(&desc).size(11.0).color(t.text_dim));
                            }
                            ui.label(egui::RichText::new(format!("成员数: {count}")).size(11.0).color(t.text_dim));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("解组").clicked() {
                                ungroup_target = Some(name.clone());
                            }
                            if ui.small_button("选择").clicked() {
                                select_target = Some(members);
                            }
                        });
                    });
                    ui.separator();
                }
            });

        if let Some(name) = ungroup_target {
            let _ = app.run("ungroup", json!({ "name": name }));
        }
        if let Some(members) = select_target {
            app.session.set_selection(members);
        }
    }

    // ------------------------------------------------------------------------
    // 特性 (Properties) 检查器
    // ------------------------------------------------------------------------
    fn render_properties(&mut self, ui: &mut Ui, app: &mut CadApp) {
        let t = Tokens::get();
        let sel = app.session.selection();
        let mut start_cmd = None;
        let mut clear_sel = false;

        ui.horizontal(|ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("特性").size(14.0).strong().color(t.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("快速选择").on_hover_text("按特性过滤选择图元").clicked() {
                    start_cmd = Some("ui.dialog.qselect");
                }
            });
        });
        ui.add_space(4.0);

        if let Some(cmd) = start_cmd {
            app.start(cmd);
        }

        let Ok(d) = app.session.doc() else {
            ui.label(egui::RichText::new("无打开的图形").color(t.text_dim));
            return;
        };

        egui::ScrollArea::vertical()
            .id_salt("sidebar_properties_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if sel.is_empty() {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("无选择对象").color(t.text_dim));
                    });
                    ui.add_space(6.0);

                    ui.label(egui::RichText::new("  默认绘图属性").size(12.0).strong().color(t.text_dim));
                    ui.separator();

                    let clayer = d.header.str("CLAYER", "0");
                    let celtype = d.header.str("CELTYPE", "ByLayer");
                    let celtscale = d.header.f64("CELTSCALE", 1.0);

                    self.prop_item(ui, "当前图层", &clayer);
                    self.prop_item(ui, "当前线型", &celtype);
                    self.prop_item(ui, "线型比例", &format!("{celtscale:.2}"));
                } else {
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(
                            egui::RichText::new(format!("已选择: {} 个图元", sel.len()))
                                .strong()
                                .color(t.accent),
                        );
                        if ui.small_button("清除选择").clicked() {
                            clear_sel = true;
                        }
                    });
                    ui.separator();

                    if let Some(first_handle) = sel.first()
                        && let Some(ent) = d.entity(*first_handle)
                    {
                        let kind_name = match &ent.kind {
                            EntityKind::Line(_) => "线条 (Line)",
                            EntityKind::Circle(_) => "圆 (Circle)",
                            EntityKind::Arc(_) => "圆弧 (Arc)",
                            EntityKind::LwPolyline(_) => "多段线 (Polyline)",
                            EntityKind::Text(_) => "单行文字 (Text)",
                            EntityKind::MText(_) => "多行文字 (MText)",
                            EntityKind::Insert(b) => {
                                ui.label(egui::RichText::new(format!("块引用: {}", b.block)).strong());
                                "块引用 (Insert)"
                            }
                            EntityKind::Point(_) => "点 (Point)",
                            EntityKind::Hatch(_) => "图案填充 (Hatch)",
                            EntityKind::Dimension(_) => "尺寸标注 (Dimension)",
                            _ => "其他图元",
                        };

                        ui.label(egui::RichText::new(format!("  类型: {kind_name}")).size(12.0).strong().color(t.text));
                        ui.separator();

                        self.prop_item(ui, "图层", &ent.common.layer);
                        self.prop_item(ui, "线型", &ent.common.linetype);
                        self.prop_item(ui, "线型比例", &format!("{:.2}", ent.common.ltscale));

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("  几何信息").size(12.0).strong().color(t.text_dim));
                        ui.separator();

                        match &ent.kind {
                            EntityKind::Line(l) => {
                                self.prop_item(ui, "起点 X", &format!("{:.4}", l.a.x));
                                self.prop_item(ui, "起点 Y", &format!("{:.4}", l.a.y));
                                self.prop_item(ui, "端点 X", &format!("{:.4}", l.b.x));
                                self.prop_item(ui, "端点 Y", &format!("{:.4}", l.b.y));
                                let dx = l.b.x - l.a.x;
                                let dy = l.b.y - l.a.y;
                                let len = (dx * dx + dy * dy).sqrt();
                                self.prop_item(ui, "长度", &format!("{:.4}", len));
                            }
                            EntityKind::Circle(c) => {
                                self.prop_item(ui, "圆心 X", &format!("{:.4}", c.center.x));
                                self.prop_item(ui, "圆心 Y", &format!("{:.4}", c.center.y));
                                self.prop_item(ui, "半径", &format!("{:.4}", c.radius));
                                self.prop_item(ui, "直径", &format!("{:.4}", c.radius * 2.0));
                                self.prop_item(ui, "面积", &format!("{:.4}", std::f64::consts::PI * c.radius * c.radius));
                            }
                            EntityKind::Arc(a) => {
                                self.prop_item(ui, "圆心 X", &format!("{:.4}", a.center.x));
                                self.prop_item(ui, "圆心 Y", &format!("{:.4}", a.center.y));
                                self.prop_item(ui, "半径", &format!("{:.4}", a.radius));
                                self.prop_item(ui, "起始角", &format!("{:.2}°", a.start.to_degrees()));
                                self.prop_item(ui, "终止角", &format!("{:.2}°", a.end.to_degrees()));
                            }
                            EntityKind::LwPolyline(p) => {
                                self.prop_item(ui, "顶点数量", &format!("{}", p.vertices.len()));
                                self.prop_item(ui, "是否闭合", if p.closed { "是" } else { "否" });
                            }
                            _ => {}
                        }
                    }
                }
            });

        if clear_sel {
            app.session.set_selection(Vec::new());
        }
    }

    fn prop_item(&self, ui: &mut Ui, label: &str, val: &str) {
        let t = Tokens::get();
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(label).size(12.0).color(t.text_dim));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(val).size(12.0).strong().color(t.text));
            });
        });
    }
}
